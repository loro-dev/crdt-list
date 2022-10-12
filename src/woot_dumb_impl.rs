use std::collections::HashSet;

use crate::{
    crdt::ListCrdt,
    dumb_common::{Container, Cursor, Iter, Op, OpId, OpSetImpl},
    test::TestFramework,
    woot,
};
use rand::Rng;

impl WootImpl {
    fn container_contains(
        container: &<Self as ListCrdt>::Container,
        op_id: Option<<Self as ListCrdt>::OpId>,
    ) -> bool {
        if op_id.is_none() {
            return true;
        }

        let op_id = op_id.unwrap();
        container.content.iter().any(|x| x.id == op_id)

        // if container.version_vector.len() <= op_id.client_id {
        //     return false;
        // }

        // container.version_vector[op_id.client_id] > op_id.clock
    }
}

pub struct WootImpl;
impl ListCrdt for WootImpl {
    type OpUnit = Op;

    type OpId = OpId;

    type Container = Container;

    type Cursor<'a> = Cursor<'a>;

    type Set = OpSetImpl;

    type Iterator<'a> = Iter<'a>;

    fn iter(
        container: &mut Self::Container,
        from: Option<Self::OpId>,
        to: Option<Self::OpId>,
    ) -> Self::Iterator<'_> {
        Iter {
            arr: &mut container.content,
            index: 0,
            start: from,
            end: to,
            done: false,
            started: false,
        }
    }

    fn insert_at(container: &mut Self::Container, op: Self::OpUnit, pos: usize) {
        container.content.insert(pos, op);
    }

    fn id(op: &Self::OpUnit) -> Self::OpId {
        op.id
    }

    fn cmp_id(op_a: &Self::OpUnit, op_b: &Self::OpUnit) -> std::cmp::Ordering {
        op_a.id
            .client_id
            .cmp(&op_b.id.client_id)
            .then(op_a.id.clock.cmp(&op_b.id.clock))
    }

    fn contains(op: &Self::OpUnit, id: Self::OpId) -> bool {
        op.id == id
    }

    fn integrate(container: &mut Self::Container, op: Self::OpUnit) {
        let id = Self::id(&op);
        for _ in container.version_vector.len()..id.client_id + 1 {
            container.version_vector.push(0);
        }
        assert!(container.version_vector[id.client_id] == id.clock);
        woot::integrate::<WootImpl>(container, op.clone(), op.left, op.right);

        container.version_vector[id.client_id] = id.clock + 1;
    }

    fn can_integrate(container: &Self::Container, op: &Self::OpUnit) -> bool {
        Self::container_contains(container, op.left)
            && Self::container_contains(container, op.right)
            && (op.id.clock == 0
                || Self::container_contains(
                    container,
                    Some(OpId {
                        client_id: op.id.client_id,
                        clock: op.id.clock - 1,
                    }),
                ))
    }

    fn len(container: &Self::Container) -> usize {
        container.content.len()
    }
}

impl woot::Woot for WootImpl {
    fn left(op: &Self::OpUnit) -> Option<Self::OpId> {
        op.left
    }

    fn right(op: &Self::OpUnit) -> Option<Self::OpId> {
        op.right
    }

    fn get_pos_of(container: &Container, op_id: Self::OpId) -> usize {
        container
            .content
            .iter()
            .position(|x| x.id == op_id)
            .unwrap()
    }
}

impl TestFramework for WootImpl {
    fn is_content_eq(a: &Self::Container, b: &Self::Container) -> bool {
        a.content.eq(&b.content)
    }

    fn new_container(id: usize) -> Self::Container {
        Container {
            id,
            version_vector: vec![0; 10],
            ..Default::default()
        }
    }

    fn new_op(_rng: &mut impl Rng, container: &mut Self::Container, pos: usize) -> Self::OpUnit {
        let insert_pos = pos % (container.content.len() + 1);
        let (left, right) = if container.content.is_empty() {
            (None, None)
        } else if insert_pos == 0 {
            (None, Some(container.content[0].id))
        } else if insert_pos == container.content.len() {
            (Some(container.content[insert_pos - 1].id), None)
        } else {
            (
                Some(container.content[insert_pos - 1].id),
                Some(container.content[insert_pos].id),
            )
        };

        let ans = Op {
            id: OpId {
                client_id: container.id,
                clock: container.max_clock,
            },
            left,
            right,
            deleted: false,
        };

        container.max_clock += 1;
        ans
    }

    type DeleteOp = HashSet<Self::OpId>;

    fn new_del_op(container: &Self::Container, mut pos: usize, mut len: usize) -> Self::DeleteOp {
        let content_len = container.content.real_len();
        let mut deleted = HashSet::new();
        if content_len == 0 {
            return deleted;
        }

        pos %= content_len;
        len = std::cmp::min(len, content_len - pos);
        for op in container.content.iter_real().skip(pos).take(len) {
            deleted.insert(op.id);
        }

        deleted
    }

    fn integrate_delete_op(container: &mut Self::Container, delete_set: Self::DeleteOp) {
        for op in container.content.iter_real_mut() {
            if delete_set.contains(&op.id) {
                op.deleted = true;
            }
        }
    }
}

#[cfg(test)]
mod woot_impl_test {
    use super::*;

    #[test]
    fn run() {
        for i in 0..100 {
            crate::test::test::<WootImpl>(i, 2, 1000);
        }
    }

    #[test]
    fn run3() {
        for seed in 0..100 {
            crate::test::test::<WootImpl>(seed, 3, 1000);
        }
    }

    #[test]
    fn run_n() {
        for n in 2..10 {
            crate::test::test::<WootImpl>(123, n, 10000);
        }
    }

    use crate::test::Action::*;
    #[test]
    fn issue_del() {
        crate::test::test_with_actions::<WootImpl>(
            5,
            100,
            vec![
                Delete {
                    client_id: 15336116641672254676,
                    pos: 15336116641672254676,
                    len: 15336116641672254676,
                },
                Delete {
                    client_id: 15336116641672254676,
                    pos: 15336116641672254676,
                    len: 16999940517776381140,
                },
                Delete {
                    client_id: 15336116641672260587,
                    pos: 15336116641672254676,
                    len: 15336116641672254676,
                },
                Delete {
                    client_id: 9571509118638019796,
                    pos: 16999940615146079364,
                    len: 1446803443202911211,
                },
                Sync {
                    from: 18446744073709551615,
                    to: 16999962371294232575,
                },
                Delete {
                    client_id: 16999940616948018155,
                    pos: 16999940616948018155,
                    len: 17005592192949660651,
                },
                Delete {
                    client_id: 16999940616948018155,
                    pos: 16999940616948018155,
                    len: 18446744073372691435,
                },
                Delete {
                    client_id: 16999940702848412651,
                    pos: 16999940616948018155,
                    len: 16999940616948018155,
                },
                Delete {
                    client_id: 16999962693416774635,
                    pos: 16999940616948018155,
                    len: 9548903258742582251,
                },
                Delete {
                    client_id: 18446744073709551615,
                    pos: 18446744073709551615,
                    len: 18446744073709551615,
                },
                Delete {
                    client_id: 18446744073709551615,
                    pos: 18446744073709551615,
                    len: 18446743103046942719,
                },
                Delete {
                    client_id: 18411986881299349503,
                    pos: 4294967295,
                    len: 9548902812537061376,
                },
                NewOp {
                    client_id: 9548902814626120836,
                    pos: 16999940616948018048,
                },
                Sync {
                    from: 18446743060434129940,
                    to: 18446744073709551615,
                },
                Delete {
                    client_id: 16999940616948607979,
                    pos: 16999940616948018155,
                    len: 16981926218438536171,
                },
                Delete {
                    client_id: 16999940616948018155,
                    pos: 16999940616948018155,
                    len: 16999940616948018155,
                },
                Delete {
                    client_id: 16999940616949328895,
                    pos: 16999940616948023275,
                    len: 16999940616948018155,
                },
                Delete {
                    client_id: 18441092497706576875,
                    pos: 16999940616949334015,
                    len: 15625477333024561368,
                },
                Delete {
                    client_id: 15625477333024561368,
                    pos: 15625477333024561368,
                    len: 15625477333024561368,
                },
                Delete {
                    client_id: 15625477333024561368,
                    pos: 15625477333024561368,
                    len: 15625477333024561368,
                },
                Delete {
                    client_id: 15625477333024561368,
                    pos: 16999940616946768088,
                    len: 16999940616948018155,
                },
                Delete {
                    client_id: 16999940616948018155,
                    pos: 16999962607517433855,
                    len: 16999940702847364075,
                },
                Delete {
                    client_id: 16999940616948018155,
                    pos: 16999940616948018155,
                    len: 16999962693416774635,
                },
                Delete {
                    client_id: 15625477333024561368,
                    pos: 15625477333024561368,
                    len: 15625477333024561368,
                },
                Delete {
                    client_id: 15625477333024561368,
                    pos: 15625477333024561368,
                    len: 15625477333024561368,
                },
                Delete {
                    client_id: 15625477333024561368,
                    pos: 15625477333024561368,
                    len: 15625477333024561194,
                },
                Delete {
                    client_id: 15625477333024561368,
                    pos: 16999940535023622360,
                    len: 16999940616948018155,
                },
                Delete {
                    client_id: 6762414324905410559,
                    pos: 15625477333024561368,
                    len: 15337246956872849624,
                },
                Delete {
                    client_id: 15336116641672254676,
                    pos: 15336116641672254676,
                    len: 15336116641672254676,
                },
                Delete {
                    client_id: 3110629260244866260,
                    pos: 15336115938746116907,
                    len: 15336116641672254676,
                },
                Delete {
                    client_id: 15336116641672254676,
                    pos: 9549038584906175700,
                    len: 9548902814626120836,
                },
                NewOp {
                    client_id: 6594541459071075460,
                    pos: 9511602982165447812,
                },
                NewOp {
                    client_id: 18446607800915494020,
                    pos: 72056692078084095,
                },
                Sync {
                    from: 9548902547289800704,
                    to: 132,
                },
                Sync { from: 0, to: 0 },
                Sync { from: 0, to: 0 },
                Sync { from: 0, to: 0 },
                Sync { from: 0, to: 0 },
                Sync {
                    from: 0,
                    to: 9548902814626086912,
                },
                NewOp {
                    client_id: 9548902814626120836,
                    pos: 9548902775988192388,
                },
                NewOp {
                    client_id: 9548774171765671044,
                    pos: 9548902814626120836,
                },
                Delete {
                    client_id: 149533581377791,
                    pos: 9548895079238501376,
                    len: 9548902814626120836,
                },
                Delete {
                    client_id: 1445914878893878251,
                    pos: 18446484675201602580,
                    len: 18446744073709551615,
                },
                Delete {
                    client_id: 16999940617099013099,
                    pos: 16999940616948018155,
                    len: 16999939796609264619,
                },
                Delete {
                    client_id: 16999940616948018175,
                    pos: 16999940616948018155,
                    len: 9548903344978783211,
                },
                NewOp {
                    client_id: 9548902814626120836,
                    pos: 9537362340580983940,
                },
                NewOp {
                    client_id: 9548902814626086912,
                    pos: 18446743541393949828,
                },
                Sync {
                    from: 285278207,
                    to: 9548902813581838336,
                },
                NewOp {
                    client_id: 9548902814626120836,
                    pos: 9548902814626120836,
                },
                NewOp {
                    client_id: 9548902814626120836,
                    pos: 9548902812663186564,
                },
                NewOp {
                    client_id: 72057594037896324,
                    pos: 9513854212820172800,
                },
                NewOp {
                    client_id: 9548902814626120836,
                    pos: 16999940615213188228,
                },
                Delete {
                    client_id: 1446803456761533456,
                    pos: 18446744073709551615,
                    len: 16999962371294232575,
                },
                Delete {
                    client_id: 16999940616948018155,
                    pos: 16999940616948018155,
                    len: 16999940616948018155,
                },
                Delete {
                    client_id: 16999940616948023275,
                    pos: 16999940616948018175,
                    len: 16999940616948018155,
                },
                Delete {
                    client_id: 18446721997240789995,
                    pos: 16999940616948023295,
                    len: 18446744073372691435,
                },
                Sync {
                    from: 18446744073709497855,
                    to: 18446744073709551615,
                },
                Sync {
                    from: 1085102842656147785,
                    to: 18446744073709489935,
                },
                Delete {
                    client_id: 18446744073709551615,
                    pos: 18446479156084473855,
                    len: 18446744073709551615,
                },
                Delete {
                    client_id: 18374686479671623680,
                    pos: 10055284024483512320,
                    len: 10055284024492657547,
                },
                NewOp {
                    client_id: 10055284024492657547,
                    pos: 10055284024492657547,
                },
                NewOp {
                    client_id: 18374968553989966731,
                    pos: 18446742978492891136,
                },
                NewOp {
                    client_id: 18392136833011023871,
                    pos: 1114367,
                },
                NewOp {
                    client_id: 9548902814626120836,
                    pos: 18374967424295666820,
                },
                Sync {
                    from: 5046283382468640785,
                    to: 17005592192949676870,
                },
                Delete {
                    client_id: 16999940616948018155,
                    pos: 18411986881291283435,
                    len: 18446744073441116159,
                },
                Delete {
                    client_id: 18446744073709551615,
                    pos: 18446744073709551615,
                    len: 18446744073709551615,
                },
                Delete {
                    client_id: 18446744073709551615,
                    pos: 589823,
                    len: 18446744073709486080,
                },
                Delete {
                    client_id: 18446744073709551615,
                    pos: 18446744073709551615,
                    len: 18446744073709551615,
                },
            ],
        )
    }

    use ctor::ctor;
    #[ctor]
    fn init_color_backtrace() {
        color_backtrace::install();
    }
}
