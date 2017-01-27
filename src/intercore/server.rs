
use queues::publisher::{Publisher, Subscriber};
use intercore::bus::{Channel, send};
use intercore::message::{Message, AckPub, AckSub};
use reactors::cps::CpsTask;
use reactors::job::Job;
use reactors::task::{Task, Context, Termination};
use reactors::scheduler::Scheduler;
use handle::{from_raw, into_raw, use_};

// The Server of InterCore protocol is handled in Scheduler context

pub fn handle_intercore<'a>(sched: &'a mut Scheduler<'a>,
                            message: Option<&'a Message>,
                            bus: &'a Channel,
                            s: &'a Subscriber<Message>)
                            -> Context<'a> {

    // println!("{:?}", s);

    match message {

        Some(&Message::Spawn(ref v)) if v.to == bus.id => {
            println!("InterCore Spawn {:?} {:?}", bus.id, v);
            let x = into_raw(sched);
            from_raw(x).spawn(Job::Cps(CpsTask::new(sched.mem())),
                              Termination::Recursive,
                              Some(&v.txt));
            Context::Nil
        }

        Some(&Message::QoS(task, bus, io)) => {
            println!("InterCore QoS {:?} {:?} {:?}", task, bus, io);
            Context::Nil
        }

        Some(&Message::Exec(ref task, ref cmd)) if 0 == bus.id => {
            let mut t = into_raw(sched.tasks.get_mut(task.clone()).expect("no shell"));
            from_raw(t).0.exec(Some(cmd));
            let x = from_raw(t).0.poll(Context::Nil, use_(sched));
            println!("InterCore Exec {:?} {:?} {:?}", task, cmd, x);
            Context::Nil
        }

        Some(&Message::Pub(ref p)) if p.to == p.from && p.to == bus.id => {
            println!("Local Pub {:?} {:?}", bus.id, p);
            sched.queues.publishers().push(Publisher::with_capacity(p.cap));
            Context::NodeAck(p.task_id, use_(sched).queues.publishers().len() - 1)
        }

        Some(&Message::Sub(ref sb)) if sb.to == sb.from && sb.to == bus.id => {
            println!("Local Sub {:?} {:?}", bus.id, sb);
            let mut sub_index = None;
            if let Some(p) = sched.queues.publishers().get_mut(sb.pub_id as usize) {
                let subscriber = p.subscribe();
                // send bit just for test
                if let Some(slot) = p.next() {
                    *slot = 42;
                    p.commit();
                }
                {
                    let subs = sched.queues.subscribers();
                    subs.push(subscriber);
                    sub_index = Some(subs.len() - 1);
                }
            }
            if let Some(idx) = sub_index {
                let h = into_raw(sched);
                let mut t = from_raw(h).tasks.get_mut(sb.task_id).expect("no task");
                t.0.poll(Context::NodeAck(sb.task_id, idx), from_raw(h));
            }
            Context::Nil
        }

        Some(&Message::Pub(ref p)) if p.to == bus.id => {
            let mut pb = Publisher::with_capacity(p.cap);
            sched.queues.publishers().push(pb);
            println!("InterCore Pub {:?} {:?} {:?}", s.token, bus.id, p);
            send(bus,
                 Message::AckPub(AckPub {
                     from: bus.id,
                     to: p.from,
                     task_id: p.task_id,
                     result_id: sched.queues.publishers().len() - 1,
                 }));
            Context::Nil
        }

        Some(&Message::AckPub(ref a)) if a.to == bus.id => {
            println!("InterCore AckPub {:?} {:?}", bus.id, a);
            let h = into_raw(sched);
            let mut t = from_raw(h).tasks.get_mut(a.task_id).expect("no task");
            t.0.poll(Context::NodeAck(a.task_id, a.result_id), from_raw(h));
            Context::Nil
        }

        Some(&Message::Sub(ref sb)) if sb.to == bus.id => {
            println!("InterCore Sub {:?} {:?}", bus.id, sb);
            let pubs = sched.queues.publishers();
            if sb.pub_id < pubs.len() {
                if let Some(p) = pubs.get_mut(sb.pub_id as usize) {
                    let subscriber = p.subscribe();
                    // send bit just for test
                    if let Some(slot) = p.next() {
                        *slot = 42;
                        p.commit();
                    }

                    let message = Message::AckSub(AckSub {
                        from: bus.id,
                        to: sb.from,
                        task_id: sb.task_id,
                        result_id: subscriber.token,
                        s: subscriber,
                    });
                    send(bus, message);
                }
            }
            Context::Nil
        }

        Some(&Message::AckSub(ref a)) if a.to == bus.id => {
            println!("InterCore AckSub {:?} {:?}", bus.id, a);
            let sub_index;
            {
                let subs = sched.queues.subscribers();
                subs.push(a.s.clone());
                sub_index = subs.len() - 1;
            }
            let h = into_raw(sched);
            let mut t = from_raw(h).tasks.get_mut(a.task_id).expect("no task");
            t.0.poll(Context::NodeAck(a.task_id, sub_index), from_raw(h));
            Context::Nil
        }
        Some(x) => {
            println!("Test {:?}", x);
            Context::Nil
        }
        None => Context::Nil,
    }
}
