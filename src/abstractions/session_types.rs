#![cfg_attr(feature = "chan_select", feature(mpsc_select))]

use std::marker;
use std::thread::spawn;
use std::mem::transmute;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::marker::PhantomData;

#[cfg(feature = "chan_select")]
use std::sync::mpsc::Select;
#[cfg(feature = "chan_select")]
use std::collections::HashMap;

#[must_use]
pub struct Chan<E, P> (Sender<Box<u8>>, Receiver<Box<u8>>, PhantomData<(E, P)>);

unsafe fn write_chan<A: marker::Send + 'static, E, P>
    (&Chan(ref tx, _, _): &Chan<E, P>, x: A)
{
    let tx: &Sender<Box<A>> = transmute(tx);
    tx.send(Box::new(x)).unwrap();
}

unsafe fn read_chan<A: marker::Send + 'static, E, P>
    (&Chan(_, ref rx, _): &Chan<E, P>) -> A
{
    let rx: &Receiver<Box<A>> = transmute(rx);
    *rx.recv().unwrap()
}

#[allow(missing_copy_implementations)]
pub struct Z;
pub struct S<N> ( PhantomData<N> );
#[allow(missing_copy_implementations)]
pub struct Eps;

pub struct Recv<A, P> ( PhantomData<(A, P)> );
pub struct Send<A, P> ( PhantomData<(A, P)> );
pub struct Choose<P, Q> ( PhantomData<(P, Q)> );
pub struct Offer<P, Q> ( PhantomData<(P, Q)> );
pub struct Rec<P> ( PhantomData<P> );
pub struct Var<N> ( PhantomData<N> );

pub unsafe trait HasDual {
    type Dual;
}

unsafe impl HasDual for Eps {
    type Dual = Eps;
}

unsafe impl <A, P: HasDual> HasDual for Send<A, P> {
    type Dual = Recv<A, P::Dual>;
}

unsafe impl <A, P: HasDual> HasDual for Recv<A, P> {
    type Dual = Send<A, P::Dual>;
}

unsafe impl <P: HasDual, Q: HasDual> HasDual for Choose<P, Q> {
    type Dual = Offer<P::Dual, Q::Dual>;
}

unsafe impl <P: HasDual, Q: HasDual> HasDual for Offer<P, Q> {
    type Dual = Choose<P::Dual, Q::Dual>;
}

unsafe impl HasDual for Var<Z> {
    type Dual = Var<Z>;
}

unsafe impl <N> HasDual for Var<S<N>> {
    type Dual = Var<S<N>>;
}

unsafe impl <P: HasDual> HasDual for Rec<P> {
    type Dual = Rec<P::Dual>;
}

pub enum Branch<L, R> {
    Left(L),
    Right(R)
}

impl <E, P> Drop for Chan<E, P> {
    fn drop(&mut self) {
        panic!("Session channel prematurely dropped");
    }
}

impl<E> Chan<E, Eps> {
    pub fn close(mut self) {
        use std::mem;
        let mut sender = unsafe { mem::uninitialized() };
        let mut receiver = unsafe { mem::uninitialized() };
        mem::swap(&mut self.0, &mut sender);
        mem::swap(&mut self.1, &mut receiver);
        drop(sender);drop(receiver); // drop them
        mem::forget(self);
    }
}

impl<E, P, A: marker::Send + 'static> Chan<E, Send<A, P>> {
    #[must_use]
    pub fn send(self, v: A) -> Chan<E, P> {
        unsafe {
            write_chan(&self, v);
            transmute(self)
        }
    }
}

impl<E, P, A: marker::Send + 'static> Chan<E, Recv<A, P>> {
    #[must_use]
    pub fn recv(self) -> (Chan<E, P>, A) {
        unsafe {
            let v = read_chan(&self);
            (transmute(self), v)
        }
    }
}

impl<E, P, Q> Chan<E, Choose<P, Q>> {
    #[must_use]
    pub fn sel1(self) -> Chan<E, P> {
        unsafe {
            write_chan(&self, true);
            transmute(self)
        }
    }

    #[must_use]
    pub fn sel2(self) -> Chan<E, Q> {
        unsafe {
            write_chan(&self, false);
            transmute(self)
        }
    }
}

impl<Z, A, B> Chan<Z, Choose<A, B>> {
    #[must_use]
    pub fn skip(self) -> Chan<Z, B> {
        self.sel2()
    }
}

impl<Z, A, B, C> Chan<Z, Choose<A, Choose<B, C>>> {
    #[must_use]
    pub fn skip2(self) -> Chan<Z, C> {
        self.sel2().sel2()
    }
}

impl<Z, A, B, C, D> Chan<Z, Choose<A, Choose<B, Choose<C, D>>>> {
    #[must_use]
    pub fn skip3(self) -> Chan<Z, D> {
        self.sel2().sel2().sel2()
    }
}

impl<Z, A, B, C, D, E> Chan<Z, Choose<A, Choose<B, Choose<C, Choose<D, E>>>>> {
    #[must_use]
    pub fn skip4(self) -> Chan<Z, E> {
        self.sel2().sel2().sel2().sel2()
    }
}

impl<Z, A, B, C, D, E, F> Chan<Z, Choose<A, Choose<B, Choose<C, Choose<D,
                          Choose<E, F>>>>>> {
    #[must_use]
    pub fn skip5(self) -> Chan<Z, F> {
        self.sel2().sel2().sel2().sel2().sel2()
    }
}

impl<Z, A, B, C, D, E, F, G> Chan<Z, Choose<A, Choose<B, Choose<C, Choose<D,
                             Choose<E, Choose<F, G>>>>>>> {
    #[must_use]
    pub fn skip6(self) -> Chan<Z, G> {
        self.sel2().sel2().sel2().sel2().sel2().sel2()
    }
}

impl<Z, A, B, C, D, E, F, G, H> Chan<Z, Choose<A, Choose<B, Choose<C, Choose<D,
                                        Choose<E, Choose<F, Choose<G, H>>>>>>>> {
    #[must_use]
    pub fn skip7(self) -> Chan<Z, H> {
        self.sel2().sel2().sel2().sel2().sel2().sel2().sel2()
    }
}

impl<E, P, Q> Chan<E, Offer<P, Q>> {
    #[must_use]
    pub fn offer(self) -> Branch<Chan<E, P>, Chan<E, Q>> {
        unsafe {
            let b = read_chan(&self);
            if b {
                Branch::Left(transmute(self))
            } else {
                Branch::Right(transmute(self))
            }
        }
    }
}

impl<E, P> Chan<E, Rec<P>> {
    #[must_use]
    pub fn enter(self) -> Chan<(P, E), P> {
        unsafe { transmute(self) }
    }
}

impl<E, P> Chan<(P, E), Var<Z>> {
    #[must_use]
    pub fn zero(self) -> Chan<(P, E), P> {
        unsafe { transmute(self) }
    }
}

impl<E, P, N> Chan<(P, E), Var<S<N>>> {
    #[must_use]
    pub fn succ(self) -> Chan<E, Var<N>> {
        unsafe { transmute(self) }
    }
}

#[cfg(feature = "chan_select")]
#[must_use]
pub fn hselect<E, P, A>(mut chans: Vec<Chan<E, Recv<A, P>>>)
                        -> (Chan<E, Recv<A, P>>, Vec<Chan<E, Recv<A, P>>>)
{
    let i = iselect(&chans);
    let c = chans.remove(i);
    (c, chans)
}

#[cfg(feature = "chan_select")]
pub fn iselect<E, P, A>(chans: &Vec<Chan<E, Recv<A, P>>>) -> usize {
    let mut map = HashMap::new();

    let id = {
        let sel = Select::new();
        let mut handles = Vec::with_capacity(chans.len()); // collect all the handles

        for (i, chan) in chans.iter().enumerate() {
            let &Chan(_, ref rx, _) = chan;
            let handle = sel.handle(rx);
            map.insert(handle.id(), i);
            handles.push(handle);
        }

        for handle in handles.iter_mut() { // Add
            unsafe { handle.add(); }
        }

        let id = sel.wait();

        for handle in handles.iter_mut() { // Clean up
            unsafe { handle.remove(); }
        }

        id
    };
    map.remove(&id).unwrap()
}

#[cfg(feature = "chan_select")]
pub struct ChanSelect<'c, T> {
    chans: Vec<(&'c Chan<(), ()>, T)>,
}

#[cfg(feature = "chan_select")]
impl<'c, T> ChanSelect<'c, T> {
    pub fn new() -> ChanSelect<'c, T> {
        ChanSelect {
            chans: Vec::new()
        }
    }

    pub fn add_recv_ret<E, P, A: marker::Send>(&mut self,
                                               chan: &'c Chan<E, Recv<A, P>>,
                                               ret: T)
    {
        self.chans.push((unsafe { transmute(chan) }, ret));
    }

    pub fn add_offer_ret<E, P, Q>(&mut self,
                                  chan: &'c Chan<E, Offer<P, Q>>,
                                  ret: T)
    {
        self.chans.push((unsafe { transmute(chan) }, ret));
    }

    pub fn wait(self) -> T {
        let sel = Select::new();
        let mut handles = Vec::with_capacity(self.chans.len());
        let mut map = HashMap::new();

        for (chan, ret) in self.chans.into_iter() {
            let &Chan(_, ref rx, _) = chan;
            let h = sel.handle(rx);
            let id = h.id();
            map.insert(id, ret);
            handles.push(h);
        }

        for handle in handles.iter_mut() {
            unsafe { handle.add(); }
        }

        let id = sel.wait();

        for handle in handles.iter_mut() {
            unsafe { handle.remove(); }
        }
        map.remove(&id).unwrap()
    }

    pub fn len(&self) -> usize {
        self.chans.len()
    }
}

#[cfg(feature = "chan_select")]
impl<'c> ChanSelect<'c, usize> {
    pub fn add_recv<E, P, A: marker::Send>(&mut self,
                                           c: &'c Chan<E, Recv<A, P>>)
    {
        let index = self.chans.len();
        self.add_recv_ret(c, index);
    }

    pub fn add_offer<E, P, Q>(&mut self,
                              c: &'c Chan<E, Offer<P, Q>>)
    {
        let index = self.chans.len();
        self.add_offer_ret(c, index);
    }
}

#[must_use]
pub fn session_channel<P: HasDual>() -> (Chan<(), P>, Chan<(), P::Dual>) {
    let (tx1, rx1) = channel();
    let (tx2, rx2) = channel();

    let c1 = Chan(tx1, rx2, PhantomData);
    let c2 = Chan(tx2, rx1, PhantomData);

    (c1, c2)
}

pub fn connect<F1, F2, P>(srv: F1, cli: F2)
    where F1: Fn(Chan<(), P>) + marker::Send + 'static,
          F2: Fn(Chan<(), P::Dual>) + marker::Send,
          P: HasDual + marker::Send + 'static,
          <P as HasDual>::Dual: HasDual + marker::Send + 'static
{
    let (c1, c2) = session_channel();
    let t = spawn(move || srv(c1));
    cli(c2);
    t.join().unwrap();
}

#[macro_export]
macro_rules! offer {
    (
        $id:ident, $branch:ident => $code:expr, $($t:tt)+
    ) => (
        match $id.offer() {
            Branch::Left($id) => $code,
            Branch::Right($id) => offer!{ $id, $($t)+ }
        }
    );
    (
        $id:ident, $branch:ident => $code:expr
    ) => (
        $code
    )
}

#[cfg(features = "chan_select")]
#[macro_export]
macro_rules! chan_select {
    (
        $(($c:ident, $name:pat) = $rx:ident.recv() => $code:expr),+
    ) => ({
        let index = {
            let mut sel = $crate::ChanSelect::new();
            $( sel.add_recv(&$rx); )+
            sel.wait()
        };
        let mut i = 0;
        $( if index == { i += 1; i - 1 } { let ($c, $name) = $rx.recv(); $code }
           else )+
        { unreachable!() }
    });

    (
        $($res:ident = $rx:ident.offer() => { $($t:tt)+ }),+
    ) => ({
        let index = {
            let mut sel = $crate::ChanSelect::new();
            $( sel.add_offer(&$rx); )+
            sel.wait()
        };
        let mut i = 0;
        $( if index == { i += 1; i - 1 } { $res = offer!{ $rx, $($t)+ } } else )+
        { unreachable!() }
    })
}
