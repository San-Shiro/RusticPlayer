use cargo_metadata::MetadataCommand;

#[test]
fn test_workspace_dependency_architecture() {
    let metadata = MetadataCommand::new()
        .exec()
        .expect("Failed to execute cargo metadata");

    let workspace_members: std::collections::HashSet<_> =
        metadata.workspace_members.iter().cloned().collect();

    let resolve = metadata
        .resolve
        .expect("Failed to resolve cargo metadata graph");

    let pkg_name_by_id: std::collections::HashMap<_, _> = metadata
        .packages
        .iter()
        .map(|p| (p.id.clone(), p.name.clone()))
        .collect();

    let mut workspace_deps: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for node in &resolve.nodes {
        if workspace_members.contains(&node.id) {
            let name = &pkg_name_by_id[&node.id];
            let deps: Vec<String> = node
                .deps
                .iter()
                .filter(|dep| workspace_members.contains(&dep.pkg))
                .map(|dep| pkg_name_by_id[&dep.pkg].clone())
                .collect();
            workspace_deps.insert(name.clone(), deps);
        }
    }

    // Invariant 1: rustic_core has zero workspace dependencies
    let core_deps = workspace_deps.get("rustic_core").cloned().unwrap_or_default();
    assert!(
        core_deps.is_empty(),
        "rustic_core must have zero workspace dependencies, found: {:?}",
        core_deps
    );

    // Invariant 2: sources, services, jam, platform may depend on rustic_core ONLY
    let core_only_crates = ["sources", "services", "jam", "platform"];
    for crate_name in &core_only_crates {
        if let Some(deps) = workspace_deps.get(*crate_name) {
            for dep in deps {
                assert_eq!(
                    dep, "rustic_core",
                    "Crate '{}' must depend on 'rustic_core' only, but depends on '{}'",
                    crate_name, dep
                );
            }
        }
    }

    // Invariant 3: ui-kit has no edge to engine or rustic_core
    if let Some(deps) = workspace_deps.get("ui-kit") {
        assert!(
            !deps.contains(&"engine".to_string()) && !deps.contains(&"rustic_core".to_string()),
            "ui-kit must not depend on engine or rustic_core, found: {:?}",
            deps
        );
    }

    // Invariant 4: lower level engine crates (audio, library, sources, services, platform, jam) are depended on only by engine (or cli/app)
    let lower_level = ["audio", "library", "sources", "services", "platform", "jam"];
    for (pkg_name, deps) in &workspace_deps {
        if pkg_name != "engine" && pkg_name != "cli" && pkg_name != "app" {
            for lower in &lower_level {
                assert!(
                    !deps.contains(&lower.to_string()),
                    "Crate '{}' is not allowed to depend on lower-level crate '{}'",
                    pkg_name,
                    lower
                );
            }
        }
    }
}
