use crate::executors::{parse_sql_author_spec_xml, parse_surface_bundle_xml};

#[test]
fn parses_surface_bundle_xml_contract() {
    let bundle = match parse_surface_bundle_xml(
        r"
        <surface_bundle>
          <project_root>/tmp/project</project_root>
          <catalog_table_name>wendao_sql_tables</catalog_table_name>
          <column_catalog_table_name>wendao_sql_columns</column_catalog_table_name>
          <view_source_catalog_table_name>wendao_sql_view_sources</view_source_catalog_table_name>
          <policy>
            <max_limit>8</max_limit>
            <allowed_op>eq</allowed_op>
            <allowed_op>contains</allowed_op>
            <require_filter_for>repo_content_chunk</require_filter_for>
          </policy>
          <objects>
            <object>
              <name>repo_entity</name>
              <kind>view</kind>
              <scope>request</scope>
              <corpus>repo</corpus>
              <source_count>1</source_count>
              <columns>
                <column>
                  <name>path</name>
                  <data_type>Utf8</data_type>
                  <nullable>false</nullable>
                  <ordinal_position>1</ordinal_position>
                  <origin_kind>logical</origin_kind>
                </column>
              </columns>
            </object>
          </objects>
        </surface_bundle>
        ",
    ) {
        Ok(bundle) => bundle,
        Err(error) => panic!("surface bundle should parse: {error}"),
    };

    assert_eq!(bundle.policy.max_limit, 8);
    assert_eq!(bundle.objects.len(), 1);
    assert_eq!(bundle.objects[0].columns[0].name, "path");
}

#[test]
fn parses_sql_author_spec_xml_contract() {
    let spec = match parse_sql_author_spec_xml(
        r"
        <sql_author_spec>
          <target_object>repo_entity</target_object>
          <projection>
            <column>path</column>
            <column>title</column>
          </projection>
          <filters>
            <filter>
              <column>path</column>
              <op>contains</op>
              <value>src/</value>
            </filter>
          </filters>
          <order_by>
            <item>
              <column>path</column>
              <direction>asc</direction>
            </item>
          </order_by>
          <limit>5</limit>
          <sql_draft>SELECT path FROM repo_entity</sql_draft>
        </sql_author_spec>
        ",
    ) {
        Ok(spec) => spec,
        Err(error) => panic!("author spec should parse: {error}"),
    };

    assert_eq!(spec.target_object, "repo_entity");
    assert_eq!(spec.projection, vec!["path", "title"]);
    assert_eq!(spec.filters.len(), 1);
    assert_eq!(spec.order_by[0].direction, "asc");
}
