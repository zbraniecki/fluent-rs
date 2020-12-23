// use fluent_bundle::FluentArgs;
// use fluent_fallback::testing::MockGenerator;
// use fluent_fallback::Localization;
// use fluent_testing::get_scenarios;

// #[test]
// fn sanity() {
//     let gen = MockGenerator::default();

//     let mut loc = Localization::with_generator(vec!["foo.ftl".to_string()], true, gen);
//     let mut errors = vec![];

//     loc.add_resource_id("foo2.dtl".to_string());

//     let _ = loc.format_value_sync("key1", None, &mut errors);

//     loc.set_async();

//     let _ = loc.format_value("key1", None, &mut errors);
// }

// #[test]
// fn scenarios() {
//     let scenarios = get_scenarios();

//     for scenario in scenarios {
//         let gen = MockGenerator::new(
//             scenario
//                 .locales
//                 .iter()
//                 .map(|l| l.parse().unwrap())
//                 .collect(),
//             scenario.file_sources.clone(),
//         );

//         let loc = Localization::with_generator(scenario.res_ids.clone(), true, gen);
//         let mut errors = vec![];

//         for query in scenario.queries.iter() {
//             let args = query.input.args.as_ref().map(|args| {
//                 let mut result = FluentArgs::new();
//                 for arg in args.as_slice() {
//                     result.add(arg.id.clone(), arg.value.clone().into());
//                 }
//                 result
//             });
//             if let Some(output) = &query.output {
//                 if let Some(value) = &output.value {
//                     let v = loc.format_value_sync(&query.input.id, args.as_ref(), &mut errors);
//                     assert_eq!(v, value.as_str());
//                 }
//             }
//             assert_eq!(errors, vec![]);
//         }
//     }
// }
