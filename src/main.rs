type Stat = i8; // -10 .. 10
type Duration = u8;
type ObjId = usize;

type Pos = (i8, i8);

struct Sim {
    name: String,
    queue: Vec<Action>,
    current: Option<(Action, Duration)>,
    pos: Pos,

    food: Stat,
    sleep: Stat,
}

fn dist(a: Pos, b: Pos) -> u8 {
    let mut accum : u8 = 0;
    accum = accum.saturating_add((a.0 - b.0).abs() as u8);
    accum = accum.saturating_add((a.1 - b.1).abs() as u8);
    accum
}

impl Sim {
    fn update(&mut self, cycle: usize,
        adverts: &[(ObjId, Advert)], objs: &mut [Box<dyn Interact>])
    {
        // stat decay
        if cycle % 5 == 0 { self.food -= 1 }
        if cycle % 10 == 0 { self.sleep -= 1 }
        // enactment
        if let Some((ena, dur)) = self.current.take() {
            if dur == 0 {
                ena.finish(self, objs);
                self.current = None;
            } else {
                self.current = Some((ena, dur - 1));
            }
            return
        }
        // enact prep
        if self.current.is_none() && self.queue.len() > 0 {
            let act = self.queue.remove(0);
            let dur = act.start(self, objs);
            let _ = self.current.insert((act, dur));
        }
        // plan
        if self.food < 2 {
            // finding nearest food
            let choice = adverts.iter().fold(None, | cur, (id, ad)| {
                // choosing something
                if ad.food <= 0 { return cur }
                let obj = objs.get_mut(*id).expect("Invalid ID");
                let d = dist(obj.pos(), self.pos);
                if let Some((pid, pd)) = cur {
                    if d < pd { return Some((id, d)); }
                    return Some((pid, pd));
                } else {
                    return Some((id, d));
                }
            });
            if let Some((id, _)) = choice {
                let obj = objs.get_mut(*id).expect("Invalid ID").pos();
                self.queue.push(Action::Goto(obj));
                self.queue.push(Action::Interact(*id));
            }
        }
        if self.sleep < 0 {
            // finding nearest food
            let choice = adverts.iter().fold(None, | cur, (id, ad)| {
                // choosing something
                if ad.sleep <= 0 { return cur }
                let obj = objs.get_mut(*id).expect("Invalid ID");
                let d = dist(obj.pos(), self.pos);
                if let Some((pid, pd)) = cur {
                    if d < pd { return Some((id, d)); }
                    return Some((pid, pd));
                } else {
                    return Some((id, d));
                }
            });
            if let Some((id, _)) = choice {
                let obj = objs.get_mut(*id).expect("Invalid ID").pos();
                self.queue.push(Action::Goto(obj));
                self.queue.push(Action::Interact(*id));
            }
        }
    }
}

#[derive(Clone)]
struct Advert {
    food: Stat,
    sleep: Stat,
}

enum Action {
    Goto(Pos),
    Interact(ObjId),
}
impl Action {
    fn finish(self, sim: &mut Sim, objs: &mut [Box<dyn Interact>]) {
        match self {
            Action::Goto((x, y)) => { sim.pos = (x, y); }
            Action::Interact(id) => {
                objs.get_mut(id).expect("invalid ID")
                    .interact_finish(sim);
            }
        }
    }
    fn start(&self, sim: &mut Sim, objs: &mut [Box<dyn Interact>]) -> Duration {
        match self.clone() {
            Action::Goto((x, y)) => {
                println!("{} is walking to ({}, {})", &sim.name, x, y);
                dist(sim.pos, (*x, *y))
            }
            Action::Interact(id) => {
                objs.get_mut(*id).expect("invalid ID")
                    .interact_start(sim)
            }
        }
    }
}

trait Interact {
    fn interact_start(&mut self, sim: &mut Sim) -> Duration;
    fn interact_finish(&mut self, sim: &mut Sim);
    fn advertise(&self) -> Option<Advert>;
    fn pos(&self) -> Pos;
}

struct FoodMachine {
    active: bool,
    pos: Pos,
    food: u8,
}
impl Interact for FoodMachine {
    fn advertise(&self) -> Option<Advert> {
        if !self.active {
            return None;
        }
        Some(Advert { food: 5, sleep: 0 })
    }
    fn interact_start(&mut self, sim: &mut Sim) -> Duration {
        println!("{} is using the food machine", &sim.name);
        self.active = false;
        sim.food = (sim.food + 5).max(10);
        self.food -= 1;
        2
    }
    fn interact_finish(&mut self, sim: &mut Sim) {
        println!("{} is refreshed", &sim.name);
        if self.food > 0 {self.active = true;}
    }
    fn pos(&self) -> Pos {
        self.pos.clone()
    }
}

struct Bed {
    active: bool,
    pos: Pos,
}
impl Interact for Bed {
    fn advertise(&self) -> Option<Advert> {
        if !self.active {
            return None;
        }
        Some(Advert { food: 0, sleep: 10 })
    }
    fn interact_start(&mut self, sim: &mut Sim) -> Duration {
        println!("{} is sleeping", &sim.name);
        self.active = false;
        sim.food = (sim.sleep + 10).max(10);
        5
    }
    fn interact_finish(&mut self, sim: &mut Sim) {
        println!("{} woke up", &sim.name);
        self.active = true;
    }
    fn pos(&self) -> Pos {
        self.pos.clone()
    }
}

struct World {
    sims: Vec<Sim>,
    objs: Vec<Box<dyn Interact>>,
    adverts: Vec<(ObjId, Advert)>,
}

fn detach<'a, 'b: 'a, T: ?Sized>(x: &'a T) -> &'b T {
    use std::ptr::{read, addr_of};
    unsafe { read(addr_of!(x).cast()) }
}

impl World {
    fn view_adverts(&self) -> impl Iterator<Item = (ObjId, Advert)> + '_ {
        self.objs.iter().filter_map(|x| x.advertise()).enumerate()
    }
    fn update(&mut self, cycle: usize) {
        self.adverts.clear();
        self.adverts.extend(detach(self).view_adverts());
        for sim in self.sims.iter_mut() {
            sim.update(cycle, &self.adverts[..], &mut self.objs[..]);
        }
    }
}

fn main() {
    let mut world = World {
            sims: vec![Sim {
                name: String::from("Dave"),
                pos: (-5, 5),
                queue: Vec::new(),
                current: None,
                food: 8,
                sleep: 1,
            },
            Sim {
                name: String::from("Steve"),
                pos: (-5, 5),
                queue: Vec::new(),
                current: None,
                food: 6,
                sleep: 10,
            },
        ],
        objs: vec![
            Box::new(FoodMachine {
                active: true,
                pos: (15, 0),
                food: 80,
            }),
            Box::new(Bed {
                active: true,
                pos: (15, 15),
            }),
        ],
        adverts: Vec::new(),
    };
    for cycle in 1..=200 {
        println!("\tcycle {cycle}");
        world.update(cycle);
    }
}
