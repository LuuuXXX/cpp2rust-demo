use crate::{string, queue, priority_queue};

hicc::import_lib! {
    #![link_name = "hicc_std_queue"]
    hicc::cpp! {
        #include <hicc/std/string.hpp>
        #include <hicc/std/queue.hpp>
    }
    class string;

    /// 对应`std::queue<int>`
    pub class QueueInt = queue<hicc::Pod<i32>>;
    #[cpp(func = "std::unique_ptr<std::queue<int>> hicc::make_unique<std::queue<int>>()")]
    #[member(class = QueueInt, method = new)]
    pub fn queue_int_new() -> QueueInt;

    /// 对应`std::queue<std::string>`
    pub class QueueString = queue<string>;
    #[cpp(func = "std::unique_ptr<std::queue<std::string>> hicc::make_unique<std::queue<std::string>>()")]
    #[member(class = QueueString, method = new)]
    pub fn queue_string_new() -> QueueString;

    /// 对应`std::priority_queue<int>`
    pub class PriorityQueueInt = priority_queue<hicc::Pod<i32>>;
    #[cpp(func = "std::unique_ptr<std::priority_queue<int>> hicc::make_unique<std::priority_queue<int>>()")]
    #[member(class = PriorityQueueInt, method = new)]
    pub fn priority_queue_int_new() -> PriorityQueueInt;
}