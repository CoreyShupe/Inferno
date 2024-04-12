//! Concurrent Linked List Implementation

use std::fmt::Debug;
use std::mem::{ManuallyDrop, MaybeUninit};
use std::sync::atomic::{AtomicBool, AtomicPtr};
use std::sync::{Arc, RwLock};

use arc_swap::ArcSwap;
use crossbeam_epoch::Atomic;

pub trait LikeLinkedList: Default + Send + Sync {
    type Item;

    fn new_with(initial_value: Self::Item) -> Self;

    fn push_front(&self, value: Self::Item);

    fn push_back(&self, value: Self::Item);

    fn pop_front(&self) -> Option<Self::Item>
    where
        Self::Item: Clone;

    fn pop_back(&self) -> Option<Self::Item>
    where
        Self::Item: Clone;
}

// atomic linked list
pub struct ManualLinkedList<T> {
    head: RwLock<Option<ManualNodePtr<T>>>,
    tail: RwLock<Option<ManualNodePtr<T>>>,
}

impl<T> Default for ManualLinkedList<T> {
    fn default() -> Self {
        Self {
            head: RwLock::new(None),
            tail: RwLock::new(None),
        }
    }
}

type ManualNodePtr<T> = *mut ManuallyDrop<ManualLinkedListNode<T>>;

pub struct ManualLinkedListNode<T> {
    prior: MaybeUninit<ManualNodePtr<T>>,
    value: T,
    next: MaybeUninit<ManualNodePtr<T>>,
}

// concurrent linked list
pub struct ArcSwapLinkedList<T> {
    head: ArcSwap<Option<ArcSwapListNode<T>>>,
    tail: ArcSwap<Option<ArcSwapListNode<T>>>,
}

impl<T> Default for ArcSwapLinkedList<T> {
    fn default() -> Self {
        Self {
            head: Arc::new(None).into(),
            tail: Arc::new(None).into(),
        }
    }
}

impl<T> LikeLinkedList for ArcSwapLinkedList<T>
where
    T: Send + Sync,
{
    type Item = T;

    fn new_with(initial_value: T) -> Self {
        let inner = Arc::new(Some(ArcSwapListNode {
            prior: Arc::new(None).into(),
            value: initial_value,
            next: Arc::new(None).into(),
        }));
        let head = ArcSwap::from(inner.clone());
        Self {
            tail: ArcSwap::from(inner.clone()),
            head,
        }
    }

    fn push_front(&self, value: T) {
        let node = Arc::new(Some(ArcSwapListNode {
            prior: Arc::new(None).into(),
            value,
            next: Arc::new(None).into(),
        }));
        let old_head = self.head.swap(node.clone());
        if let Some(node) = node.as_ref() {
            node.next.store(old_head.clone());
        }

        if let Some(old_head) = old_head.as_ref() {
            old_head.prior.store(node);
        } else {
            let tail = self.tail.load();

            // swap the assumed empty tail to the new node as we're resurrecting an empty list
            let mut result = if tail.is_none() {
                self.tail.compare_and_swap(tail, node.clone())
            } else {
                tail
            };
            let mut result_clone = Arc::clone(&result);

            // we need to relink the front to the back proper node
            // since it wasn't actually None when doing the replacement
            while let Some(result_ref) = result.as_ref() {
                // get the current tail node to determine if there's more before
                let next = result_ref.prior.load();
                // no more prior, we can safely assume this is the next node
                if next.is_none() {
                    result_ref.prior.store(node.clone());
                    let Some(node) = node.as_ref() else {
                        unreachable!();
                    };
                    // store the next node as the tail for the current node to re-attach
                    node.next.store(result_clone);
                    break;
                }
                result = next;
                result_clone = Arc::clone(&result);
            }
        }
    }

    fn push_back(&self, value: T) {
        let node = Arc::new(Some(ArcSwapListNode {
            prior: Arc::new(None).into(),
            value,
            next: Arc::new(None).into(),
        }));
        let old_tail = self.tail.swap(node.clone());
        if let Some(node) = node.as_ref() {
            node.prior.store(old_tail.clone());
        }
        if let Some(old_tail) = old_tail.as_ref() {
            old_tail.next.store(node)
        } else {
            let head = self.head.load();

            // swap the assumed empty head to the new node as we're resurrecting an empty list
            let mut result = if head.is_none() {
                self.head.compare_and_swap(head, node.clone())
            } else {
                head
            };
            let mut result_clone = Arc::clone(&result);

            // we need to relink the front to the back proper node
            // since it wasn't actually None when doing the replacement
            while let Some(result_ref) = result.as_ref() {
                // get the current tail node to determine if there's more before
                let next = result_ref.next.load();
                // no more prior, we can safely assume this is the next node
                if next.is_none() {
                    result_ref.next.store(node.clone());
                    let Some(node) = node.as_ref() else {
                        unreachable!();
                    };
                    // store the next node as the tail for the current node to re-attach
                    node.prior.store(result_clone);
                    break;
                }
                result = next;
                result_clone = Arc::clone(&result);
            }
        }
    }

    fn pop_front(&self) -> Option<T>
    where
        T: Clone,
    {
        let head = self.head.load();
        let Some(head) = head.as_ref() else {
            return None;
        };

        let next = head.next.load();
        let next_clone = Arc::clone(&next);
        let old_head = if next.is_some() {
            let Some(next_ref) = next.as_ref() else {
                unreachable!()
            };

            next_ref.prior.store(Arc::new(None));

            self.head.swap(next_clone)
        } else {
            let old_head = self.head.swap(Arc::new(None));
            self.tail.store(Arc::new(None));
            old_head
        };

        if let Some(old_head) = old_head.as_ref() {
            Some(old_head.value.clone())
        } else {
            None
        }
    }

    fn pop_back(&self) -> Option<T>
    where
        T: Clone,
    {
        let tail = self.tail.load();
        let Some(tail) = tail.as_ref() else {
            return None;
        };

        let prior = tail.prior.load();
        let prior_clone = Arc::clone(&prior);
        let old_tail = if prior.is_some() {
            let Some(prior_ref) = prior.as_ref() else {
                unreachable!()
            };

            prior_ref.next.store(Arc::new(None));

            self.tail.swap(prior_clone)
        } else {
            let old_tail = self.tail.swap(Arc::new(None));
            self.head.store(Arc::new(None));
            old_tail
        };

        if let Some(old_tail) = old_tail.as_ref() {
            Some(old_tail.value.clone())
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct ArcSwapListNode<T> {
    prior: ArcSwap<Option<ArcSwapListNode<T>>>,
    value: T,
    next: ArcSwap<Option<ArcSwapListNode<T>>>,
}

#[cfg(test)]
mod tests {
    use super::{ArcSwapLinkedList, LikeLinkedList};
    use rstest::rstest;
    use std::sync::Arc;

    #[rstest::fixture]
    pub fn with_item() -> impl LikeLinkedList<Item = usize> {
        ArcSwapLinkedList::new_with(1)
    }

    #[rstest::fixture]
    pub fn as_default() -> impl LikeLinkedList<Item = usize> {
        ArcSwapLinkedList::default()
    }

    #[rstest]
    pub fn test_initial_value_single_pop(with_item: impl LikeLinkedList<Item = usize>) {
        let list = with_item;
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_back(), None);
        assert_eq!(list.pop_front(), None);
    }

    #[rstest]
    pub fn test_default_value_no_pop(as_default: impl LikeLinkedList<Item = usize>) {
        let list = as_default;
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.pop_back(), None);
    }

    #[rstest]
    pub fn test_initial_value_single_push(with_item: impl LikeLinkedList<Item = usize>) {
        let list = with_item;
        list.push_front(2);
        assert_eq!(list.pop_front(), Some(2));
        assert_eq!(list.pop_back(), Some(1));
        assert_eq!(list.pop_front(), None);
    }

    #[rstest]
    pub fn test_default_value_single_push(as_default: impl LikeLinkedList<Item = usize>) {
        let list = as_default;
        list.push_front(2);
        assert_eq!(list.pop_front(), Some(2));
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.pop_back(), None);
    }

    #[rstest]
    pub fn test_default_alternating_pop<L: LikeLinkedList<Item = usize>>(as_default: L) {
        let list = as_default;
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);
        assert_eq!(list.pop_back(), Some(1));
        assert_eq!(list.pop_back(), Some(2));
        assert_eq!(list.pop_back(), Some(3));
        let list = L::default();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), Some(2));
        assert_eq!(list.pop_front(), Some(3));
        list.push_back(1);
        list.push_front(2);
        list.push_back(3);
        assert_eq!(list.pop_front(), Some(2));
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), Some(3));
    }

    #[rstest]
    pub fn test_multi_threaded_push<L: LikeLinkedList<Item = usize> + 'static>(as_default: L) {
        let _ = as_default;
        let list = L::new_with(0);
        let list = Arc::new(list);

        let first_half_list = list.clone();
        let first_half = std::thread::spawn(move || {
            let list = first_half_list;
            for i in 1..=100 {
                println!("Pushing {}", i);
                list.push_front(i);
            }
        });

        let second_half_list = list.clone();
        let second_half = std::thread::spawn(move || {
            let list = second_half_list;
            for i in 0..=100 {
                println!("(2) Pushing {}", i);
                list.push_back(i);
            }
        });

        let ok1 = first_half.join();
        let ok2 = second_half.join();
        assert!(ok1.is_ok());
        assert!(ok2.is_ok());

        for i in 0..=100 {
            let pop = list.pop_front();
            assert_eq!(pop, Some(100 - i));
        }
        for i in 0..=100 {
            let pop = list.pop_front();
            assert_eq!(pop, Some(i));
        }
    }
}
