use std::collections::VecDeque;

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
    fn prev_page(&self, pager: Pager) -> Option<Self>;
    fn next_page(&self, pager: Pager) -> Option<Self>;
}

pub trait Id {
    fn id(&self) -> i64;
}

impl Paginator for PageInfo {
    fn get_pager<T: Id>(&self, data: &mut VecDeque<T>) -> Pager {
        let has_prev = self.cursor.is_some();
        let start = if has_prev { data.pop_front() } else { None };

        let has_next = data.len() as i64 > self.page_size;
        let end = if has_next { data.pop_back() } else { None };

        Pager {
            prev: start.map(|r| r.id()),
            next: end.map(|r| r.id()),
            total: None,
        }
    }

    fn prev_page(&self, pager: Pager) -> Option<Self> {
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

    fn next_page(&self, pager: Pager) -> Option<Self> {
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
    }
}
