
use queues::publisher::Publisher;
use intercore::bus::{Channel, send};
use intercore::message::{Message, AckPub, AckSub};
use reactors::cps::CpsTask;
use reactors::job::Job;
use reactors::task::{Task, Context, Termination, Poll, Error};
use reactors::scheduler::Scheduler;
use handle::{from_raw, into_raw, use_};

// The InterCore Delivery by Adressee

pub fn delivery<'a>(context: Context<'a>, sched: &'a mut Scheduler<'a>) -> Poll<Context<'a>, Error> {
    match context {
        Context::NodeAck(task, res) => {
            use_(sched).tasks.get_mut(task).expect("no shell").0.poll(context, sched)
        }
        Context::Node(..) => Poll::End(context),
        _ => Poll::Yield(Context::Nil)
    }
}

// InterCore State Machine

pub fn intercore<'a>(sched: &'a mut Scheduler<'a>, message: Option<&'a Message>, bus: &'a Channel) -> Context<'a> {

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
            let mut t = use_(sched).tasks.get_mut(p.task_id).expect("no task");
            let id = use_(sched).queues.publishers().len() - 1;
            Context::NodeAck(p.task_id, id)
        }

        Some(&Message::Sub(ref sb)) if sb.to == sb.from && sb.to == bus.id => {
            println!("Local Sub {:?} {:?}", bus.id, sb);
            let mut sub_index = None;
            if let Some(p) = sched.queues.publishers().get_mut(sb.pub_id as usize) {
                let subscriber = p.subscribe();
                {
                    let subs = sched.queues.subscribers();
                    subs.push(subscriber);
                    sub_index = Some(subs.len() - 1);
                }
            }
            if let Some(idx) = sub_index {
                let h = into_raw(sched);
                let mut t = from_raw(h).tasks.get_mut(sb.task_id).expect("no task");
                return Context::NodeAck(sb.task_id, idx);
            }
            Context::Nil
        }

        Some(&Message::Pub(ref p)) if p.to == bus.id => {
            let mut pb = Publisher::with_capacity(p.cap);
            sched.queues.publishers().push(pb);
            println!("InterCore Pub {:?} {:?}", bus.id, p);
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
            Context::NodeAck(a.task_id, a.result_id)
        }

        Some(&Message::Sub(ref sb)) if sb.to == bus.id => {
            println!("InterCore Sub {:?} {:?}", bus.id, sb);
            let pubs = sched.queues.publishers();
            if sb.pub_id < pubs.len() {
                if let Some(p) = pubs.get_mut(sb.pub_id as usize) {
                    let subscriber = p.subscribe();
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
            Context::NodeAck(a.task_id, sub_index)
        }

        Some(x) => {
            // println!("Test {:?}", x);
            Context::Nil
        }

        None => Context::Nil,
    }
}
