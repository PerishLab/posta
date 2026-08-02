use keel::Graph;
use keel::atom::string;
use keel::resource;

#[resource(frozen)]
pub(crate) struct Mailbox {
    #[field(string, unique)]
    tag: string,
    #[field(string)]
    created: string,
}

#[resource(frozen)]
pub(crate) struct Thread {
    #[field(string, unique)]
    tag: string,
    #[field(string)]
    created: string,
    #[relation(Mailbox, many2one, root)]
    mailbox: Mailbox,
}

#[resource(frozen)]
pub(crate) struct Letter {
    #[field(string, values = ("post", "reply"))]
    kind: string,
    #[field(string, unique, opt)]
    request: string,
    #[field(string, unique, opt)]
    cause: string,
    #[field(string)]
    content: string,
    #[field(string)]
    created: string,
    #[relation(Mailbox, many2one, root)]
    mailbox: Mailbox,
    #[relation(Thread, many2one, opt)]
    thread: Thread,
}

#[resource(frozen)]
pub(crate) struct Receipt {
    #[field(string, unique)]
    tag: string,
    #[field(string)]
    created: string,
    #[relation(Letter, one2one, root)]
    letter: Letter,
    #[relation(Thread, many2one)]
    thread: Thread,
}

#[resource(veil, frozen)]
pub(crate) struct Pending {
    #[field(string, unique)]
    cause: string,
    #[field(string)]
    thread: string,
    #[field(string)]
    content: string,
    #[field(string)]
    created: string,
}

pub(crate) fn graph() -> Graph {
    let mut graph = Graph::new();
    graph
        .plug::<Mailbox>()
        .plug::<Thread>()
        .plug::<Letter>()
        .plug::<Receipt>()
        .plug::<Pending>();
    graph
}
