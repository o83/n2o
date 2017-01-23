
use queues::publisher::{Publisher, Subscriber};
use intercore::bus::{Ctx, Channel, send};
use intercore::message::{Message, Pub, Sub, AckPub, AckSub, Spawn, AckSpawn};
use reactors::cpstask::CpsTask;
use commands::ast::{AST, Value};
use reactors::job::Job;
use reactors::task::{Task, Context};
use reactors::scheduler::{Scheduler, TaskTermination};
use handle;
use std::rc::Rc;

// The Server of InterCore protocol is handled in Scheduler context

pub fn handle_intercore<'a>(sched: &mut Scheduler<'a>,
                            message: Option<&'a Message>,
                            bus: &'a Channel) -> Context<'a> {

    match message {

        Some(&Message::Spawn(ref v)) if v.to == bus.id => {
            println!("InterCore Spawn {:?} {:?}", bus.id, v);
            handle::with_raw(sched, |h| {
                handle::from_raw(h).spawn(Job::Cps(CpsTask::new()),
                                          TaskTermination::Recursive,
                                          Some(&v.txt))
            });
            Context::Nil
        }

        Some(&Message::Pub(ref p)) if p.to == p.from && p.to == bus.id => {
            println!("Pub Request");
            sched.queues.publishers().push(Publisher::with_capacity(p.cap));
            Context::NodeAck(p.task_id, sched.queues.publishers().len())
        }

        Some(&Message::Pub(ref p)) if p.to == bus.id => {
            sched.queues.publishers().push(Publisher::with_capacity(p.cap));
            println!("InterCore Pub {:?} {:?}", bus.id, p);
            send(bus, Message::AckPub(AckPub {
                from: p.to,
                to: p.from,
                task_id: p.task_id,
                result_id: sched.queues.publishers().len(),
            }));
            Context::Nil
        }

        Some(&Message::AckPub(ref a)) if a.to == bus.id => {
            println!("InterCore AckPub {:?} {:?}", bus.id, a);
            Context::NodeAck(a.task_id, a.result_id)
        }

        Some(&Message::Sub(ref sb)) if sb.to == bus.id => {
            println!("InterCore Sub {:?} {:?}", bus.id, sb);
            let subs = sched.queues.subscribers();
            let pubs = sched.queues.publishers();
            if sb.pub_id < pubs.len() {
                if let Some(p) = pubs.get_mut(sb.pub_id as usize) {
                    let subscriber = p.subscribe();
                    let message = Message::AckSub(AckSub {
                        from: bus.id,
                        to: sb.from,
                        task_id: sb.task_id,
                        result_id: sched.queues.subscribers().len(),
//                        s: subscriber.clone(),
                    });
                    subs.push(subscriber);
                    send(bus, message);
                }
            }
            Context::Nil
        }

        Some(&Message::AckSub(ref a)) => {
            println!("InterCore AckSub {:?} {:?}", bus.id, a);
            Context::NodeAck(a.task_id, a.result_id)
        }
        None => {
            Context::Nil
        }
        Some(x) => {
            //println!("InterCore {:?} {:?}", bus.id, x);
            Context::Nil
        }
    }
}

