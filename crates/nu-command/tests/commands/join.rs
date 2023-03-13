use nu_test_support::nu;

#[test]
fn cases_where_result_is_same_between_join_types_inner() {
    do_cases_where_result_is_same_between_join_types("--inner")
}

#[test]
fn cases_where_result_differs_between_join_types_inner() {
    do_cases_where_result_differs_between_join_types("--inner")
}

#[test]
fn cases_where_result_is_same_between_join_types_left() {
    do_cases_where_result_is_same_between_join_types("--left")
}

#[test]
fn cases_where_result_is_same_between_join_types_outer() {
    do_cases_where_result_is_same_between_join_types("--outer")
}

#[test]
fn cases_where_result_differs_between_join_types_left() {
    do_cases_where_result_differs_between_join_types("--left")
}

#[test]
fn cases_where_result_is_same_between_join_types_right() {
    do_cases_where_result_is_same_between_join_types("--right")
}

#[test]
fn cases_where_result_differs_between_join_types_right() {
    do_cases_where_result_differs_between_join_types("--right")
}

#[test]
fn cases_where_result_differs_between_join_types_outer() {
    do_cases_where_result_differs_between_join_types("--outer")
}

fn do_cases_where_result_is_same_between_join_types(join_type: &str) {
    // .mode column
    // .headers on
    for ((left, right, on), expected) in [
        (("[]", "[]", "_"), "[]"),
        (("[]", "[{a: 1}]", "_"), "[]"),
        (("[{a: 1}]", "[]", "_"), "[]"),
        (("[{a: 1}]", "[{a: 1}]", "_"), "[]"),
        (("[{a: 1}]", "[{a: 1}]", "a"), "[[a]; [1]]"),
        (("[{a: 1} {a: 1}]", "[{a: 1}]", "a"), "[[a]; [1], [1]]"),
        (("[{a: 1}]", "[{a: 1} {a: 1}]", "a"), "[[a]; [1], [1]]"),
        (
            ("[{a: 1} {a: 1}]", "[{a: 1} {a: 1}]", "a"),
            "[[a]; [1], [1], [1], [1]]",
        ),
        (("[{a: 1 b: 1}]", "[{a: 1}]", "a"), "[[a, b]; [1, 1]]"),
        (("[{a: 1}]", "[{a: 1 b: 2}]", "a"), "[[a, b]; [1, 2]]"),
        (
            // create table l (a, b);
            // create table r (a, b);
            // insert into l (a, b) values (1, 1);
            // insert into r (a, b) values (1, 2);
            // select * from l inner join r on l.a = r.a;
            // a  b  a  b
            // -  -  -  -
            // 1  1  1  2
            ("[{a: 1 b: 1}]", "[{a: 1 b: 2}]", "a"),
            // ("[{a: 1 b: 1}]", " [[a, b]; [1, 2]]", "a"),
            "[[a, b, b_]; [1, 1, 2]]",
        ),
        (("[{a: 1}]", "[{a: 1 b: 1}]", "a"), "[[a, b]; [1, 1]]"),
        // (("[{a: 1}]", "[[a, b]; [1, 1]]", "a"), "[[a, b]; [1, 1]]"),
    ] {
        let expr = format!("{} | join {} {} {} | to nuon", left, right, join_type, on);
        let actual = nu!(cwd: ".", expr).out;
        assert_eq!(actual, expected);

        // Test again with streaming input (using `each` to convert the input into a ListStream)
        let to_list_stream = "each { |i| $i } | ";
        let expr = format!(
            "{} | {} join {} {} {} | to nuon",
            left, to_list_stream, right, join_type, on
        );
        let actual = nu!(cwd: ".", expr).out;
        assert_eq!(actual, expected);
    }
}

fn do_cases_where_result_differs_between_join_types(join_type: &str) {
    // .mode column
    // .headers on
    for ((left, right, on), join_types) in [
        (
            ("[]", "[{a: 1}]", "a"),
            [
                ("--inner", "[]"),
                ("--left", "[]"),
                ("--right", "[[a]; [1]]"),
                ("--outer", "[[a]; [1]]"),
            ],
        ),
        (
            ("[{a: 1}]", "[]", "a"),
            [
                ("--inner", "[]"),
                ("--left", "[[a]; [1]]"),
                ("--right", "[]"),
                ("--outer", "[[a]; [1]]"),
            ],
        ),
        (
            ("[{a: 2 b: 1}]", "[{a: 1}]", "a"),
            [
                ("--inner", "[]"),
                ("--left", "[[a, b]; [2, 1]]"),
                ("--right", "[[a, b]; [1, null]]"),
                ("--outer", "[[a, b]; [2, 1], [1, null]]"),
            ],
        ),
        (
            ("[{a: 1}]", "[{a: 2 b: 1}]", "a"),
            [
                ("--inner", "[]"),
                ("--left", "[[a, b]; [1, null]]"),
                ("--right", "[[a, b]; [2, 1]]"),
                ("--outer", "[[a, b]; [1, null], [2, 1]]"),
            ],
        ),
        (
            // create table l (a, b);
            // create table r (a, b);
            // insert into l (a, b) values (1, 2);
            // insert into r (a, b) values (2, 1);
            ("[{a: 1 b: 2}]", "[{a: 2 b: 1}]", "a"),
            [
                ("--inner", "[]"),
                ("--left", "[[a, b, b_]; [1, 2, null]]"),
                // select * from l right outer join r on l.a = r.a;
                // a  b  a  b
                // -  -  -  -
                //       2  1
                ("--right", "[[a, b, b_]; [2, null, 1]]"),
                ("--outer", "[[a, b, b_]; [1, 2, null], [2, null, 1]]"),
            ],
        ),
        (
            ("[{a: 1 b: 2}]", "[{a: 2 b: 1} {a: 1 b: 1}]", "a"),
            [
                ("--inner", "[[a, b, b_]; [1, 2, 1]]"),
                ("--left", "[[a, b, b_]; [1, 2, 1]]"),
                ("--right", "[[a, b, b_]; [2, null, 1], [1, 2, 1]]"),
                ("--outer", "[[a, b, b_]; [1, 2, 1], [2, null, 1]]"),
            ],
        ),
        (
            (
                "[{a: 1 b: 1} {a: 2 b: 2} {a: 3 b: 3}]",
                "[{a: 1 c: 1} {a: 3 c: 3}]",
                "a",
            ),
            [
                ("--inner", "[[a, b, c]; [1, 1, 1], [3, 3, 3]]"),
                ("--left", "[[a, b, c]; [1, 1, 1], [2, 2, null], [3, 3, 3]]"),
                ("--right", "[[a, b, c]; [1, 1, 1], [3, 3, 3]]"),
                ("--outer", "[[a, b, c]; [1, 1, 1], [2, 2, null], [3, 3, 3]]"),
            ],
        ),
        (
            // create table l (a, c);
            // create table r (a, b);
            // insert into l (a, c) values (1, 1), (2, 2), (3, 3);
            // insert into r (a, b) values (1, 1), (3, 3), (4, 4);
            (
                "[{a: 1 c: 1} {a: 2 c: 2} {a: 3 c: 3}]",
                "[{a: 1 b: 1} {a: 3 b: 3} {a: 4 b: 4}]",
                "a",
            ),
            [
                ("--inner", "[[a, c, b]; [1, 1, 1], [3, 3, 3]]"),
                ("--left", "[[a, c, b]; [1, 1, 1], [2, 2, null], [3, 3, 3]]"),
                // select * from l right outer join r on l.a = r.a;
                // a  c  a  b
                // -  -  -  -
                // 1  1  1  1
                // 3  3  3  3
                //       4  4
                ("--right", "[[a, c, b]; [1, 1, 1], [3, 3, 3], [4, null, 4]]"),
                (
                    "--outer",
                    "[[a, c, b]; [1, 1, 1], [2, 2, null], [3, 3, 3], [4, null, 4]]",
                ),
            ],
        ),
    ] {
        for (join_type_, expected) in join_types {
            if join_type_ == join_type {
                let expr = format!("{} | join {} {} {} | to nuon", left, right, join_type, on);
                let actual = nu!(cwd: ".", expr).out;
                assert_eq!(actual, expected);

                // Test again with streaming input (using `each` to convert the input into a ListStream)
                let to_list_stream = "each { |i| $i } | ";
                let expr = format!(
                    "{} | {} join {} {} {} | to nuon",
                    left, to_list_stream, right, join_type, on
                );
                let actual = nu!(cwd: ".", expr).out;
                assert_eq!(actual, expected);
            }
        }
    }
}

#[ignore]
#[test]
fn test_alternative_table_syntax() {
    let join_type = "--inner";
    for ((left, right, on), expected) in [
        (("[{a: 1}]", "[{a: 1}]", "a"), "[[a]; [1]]"),
        (("[{a: 1}]", "[[a]; [1]]", "a"), "[[a]; [1]]"),
        (("[[a]; [1]]", "[{a: 1}]", "a"), "[[a]; [1]]"),
        (("[[a]; [1]]", "[[a]; [1]]", "a"), "[[a]; [1]]"),
    ] {
        let expr = format!("{} | join {} {} {} | to nuon", left, right, join_type, on);
        let actual = nu!(cwd: ".", &expr).out;
        assert_eq!(actual, expected, "Expression was {}", &expr);
    }
}
