use std::collections::VecDeque;

#[derive(Debug)]
pub struct PageInfo {
    pub cursor: Option<i64>,
    pub page_size: i64,
    pub desc: bool,
}

pub struct Pager {
    pub prev: Option<i64>,
    pub next: Option<i64>,
    pub total: Option<i64>,
}

pub trait Paginator: Sized {
    fn get_pager<T: Id>(&self, data: &mut VecDeque<T>) -> Pager;
    fn prev_page(&self, pager: &Pager) -> Option<Self>;
    fn next_page(&self, pager: &Pager) -> Option<Self>;
}

pub trait Id {
    fn id(&self) -> i64;
}

impl Paginator for PageInfo {
    fn get_pager<T: Id>(&self, data: &mut VecDeque<T>) -> Pager {
        let has_prev = self.cursor.is_some();
        let prev = if has_prev {
            data.pop_front();
            data.front().map(|v| v.id())
        } else {
            None
        };

        let has_next = data.len() as i64 > self.page_size;
        let next = if has_next {
            data.pop_back();
            data.back().map(|v| v.id())
        } else {
            None
        };

        Pager {
            prev,
            next,
            total: None,
        }
    }

    // todo: It is not correct.
    fn prev_page(&self, pager: &Pager) -> Option<Self> {
        if pager.prev.is_some() {
            Some(PageInfo {
                cursor: pager.prev,
                page_size: self.page_size,
                desc: self.desc,
            })
        } else {
            None
        }
    }

    fn next_page(&self, pager: &Pager) -> Option<Self> {
        if pager.next.is_some() {
            Some(PageInfo {
                cursor: pager.next,
                page_size: self.page_size,
                desc: self.desc,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
pub mod pager_test_utils {
    use std::collections::VecDeque;

    use super::Id;

    pub struct TestId(i64);

    pub fn generate_test_ids(start: i64, end: i64) -> VecDeque<TestId> {
        (start..=end).map(TestId).collect()
    }

    // pub fn generate_test_ids_2(start: i64, end: i64) -> VecDeque<TestId> {
    //     (start..=end).map(|i| TestId(i)).collect()
    // }

    impl Id for TestId {
        fn id(&self) -> i64 {
            self.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paginator_should_work() {
        let page = PageInfo {
            cursor: None,
            page_size: 10,
            desc: false,
        };

        let mut items = pager_test_utils::generate_test_ids(1, 11);
        let pager = page.get_pager(&mut items);
        assert!(pager.prev.is_none());
        assert_eq!(pager.next, Some(10));

        let prev = page.prev_page(&pager);
        assert!(prev.is_none());

        // second page
        let page = page.next_page(&pager).unwrap();
        println!("{page:?}");
        let pager = page.get_pager(&mut pager_test_utils::generate_test_ids(10, 21));
        assert_eq!(pager.prev, Some(11));
        assert_eq!(pager.next, Some(20));

        // third page
        let page = page.next_page(&pager).unwrap();
        let pager = page.get_pager(&mut pager_test_utils::generate_test_ids(20, 25));
        assert!(pager.next.is_none());
        assert_eq!(pager.prev, Some(21));

        {
            let prev_page = page.prev_page(&pager);
            assert_eq!(prev_page.unwrap().cursor, Some(21));
        }
    }
}
