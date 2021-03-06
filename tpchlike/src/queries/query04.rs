use timely::order::TotalOrder;
use timely::dataflow::*;
use timely::dataflow::operators::probe::Handle as ProbeHandle;

use differential_dataflow::operators::*;
use differential_dataflow::operators::arrange::ArrangeBySelf;
use differential_dataflow::operators::group::GroupArranged;
use differential_dataflow::trace::Trace;
use differential_dataflow::trace::implementations::ord::OrdKeySpine as DefaultKeyTrace;
use differential_dataflow::lattice::Lattice;

use ::Collections;

// -- $ID$
// -- TPC-H/TPC-R Order Priority Checking Query (Q4)
// -- Functional Query Definition
// -- Approved February 1998
// :x
// :o
// select
//     o_orderpriority,
//     count(*) as order_count
// from
//     orders
// where
//     o_orderdate >= date ':1'
//     and o_orderdate < date ':1' + interval '3' month
//     and exists (
//         select
//             *
//         from
//             lineitem
//         where
//             l_orderkey = o_orderkey
//             and l_commitdate < l_receiptdate
//     )
// group by
//     o_orderpriority
// order by
//     o_orderpriority;
// :n -1

pub fn query<G: Scope>(collections: &mut Collections<G>) -> ProbeHandle<G::Timestamp> 
where G::Timestamp: Lattice+TotalOrder+Ord {

    let lineitems = 
    collections
        .lineitems()
        .flat_map(|l| if l.commit_date < l.receipt_date { Some(l.order_key) } else { None })
        .arrange_by_self()
        .group_arranged(|_k,_s,t| t.push(((), 1)), DefaultKeyTrace::new()); // <-- Distinct

    collections
        .orders()
        .flat_map(|o| 
            if o.order_date >= ::types::create_date(1993, 7, 1) && o.order_date < ::types::create_date(1993, 10, 1) {
                Some((o.order_key, o.order_priority))
            }
            else { None }
        )
        .join_core(&lineitems, |_k,v,_| Some(v.clone()))
        .count_total()
        .probe()
}