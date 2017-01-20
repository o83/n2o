
use queues::publisher::{Publisher, Subscriber};
use streams::intercore::ctx::{Ctx, Channel};
use streams::intercore::api::{Message, Ack};
use reactors::cpstask::CpsTask;
use reactors::job::Job;
use reactors::scheduler::{Scheduler, TaskTermination};
use handle;
use std::rc::Rc;

pub fn handle_intercore<'a>(sched: &mut Scheduler<'a>,
                            message: Option<&'a Message>,
                            bus: &'a Channel,
                            s: &'a Subscriber<Message>) {
    match message {
        Some(&Message::Spawn(ref v)) if v.to == bus.id => {
            println!("poll bus on core_{} {:?}", bus.id, v);
            handle::with_raw(sched, |h| {
                handle::from_raw(h).spawn(Job::Cps(CpsTask::new(Rc::new(Ctx::new()))),
                                          TaskTermination::Recursive,
                                          Some(&v.txt))
            });
            s.commit();
        }
        Some(&Message::Pub(ref pb)) if pb.to == bus.id => {
            println!("poll bus on core_{} {:?}", bus.id, pb);
            s.commit();
            if let Some(v) = bus.publisher.next() {
                *v = Message::Ack(Ack {
                    from: bus.id,
                    to: pb.from,
                    task_id: pb.task_id,
                    result_id: 0,
                    subs: sched.pb.subscribe(),
                });
                if let Some(v) = sched.pb.next() {
                    *v = Message::Unknown;
                    println!("send on core_{} {:?}", bus.id, v);
                    sched.pb.commit();
                }
                bus.publisher.commit();
            }
        }

        Some(&Message::Sub(ref sb)) if sb.to == bus.id => {
            // println!("poll bus on core_{} {:?}", bus.id, sb);
            s.commit();
        } 

        Some(&Message::Ack(ref a)) => {
            println!("ACK on core_");
            if let Some(v) = a.subs.recv() {
                println!("ACK on core_{} {:?}", bus.id, v);
                a.subs.commit();
            }
            s.commit();
        } 
        _ => (),
    }
}
