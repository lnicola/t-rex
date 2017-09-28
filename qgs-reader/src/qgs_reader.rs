//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use elementtree::Element;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io;
use std::error::Error;

fn read_xml(fname: &str) -> Result<Element, io::Error> {
    let file = File::open(fname)?;
    let mut reader = BufReader::new(file);
    Element::from_reader(&mut reader).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

fn pg_uri_from_qgs_ds(ds: &str) -> (String, &str) {
    let params: HashMap<&str, &str> = ds.split(' ')
        .map(|kv| kv.split('=').collect::<Vec<&str>>())
        .map(|vec| if vec.len() == 2 {
                 (vec[0], vec[1].trim_matches('\''))
             } else {
                 (vec[0], "")
             })
        .collect();

    //postgresql://[user[:password]@][netloc][:port][/dbname][?param1=value1&...]
    let mut uri = "postgresql://".to_string();
    if params.contains_key("user") {
        uri.push_str(params["user"]);
    }
    if params.contains_key("password") {
        uri.push_str(":");
        uri.push_str(params["password"]);
    }
    if params.contains_key("user") {
        uri.push_str("@");
    }
    if params.contains_key("host") {
        uri.push_str(params["host"]);
    } else {
        uri.push_str("%2Frun%2Fpostgresql"); // FIXME
    }
    if params.contains_key("port") {
        uri.push_str(":");
        uri.push_str(params["port"]);
    }
    if params.contains_key("dbname") {
        uri.push_str("/");
        uri.push_str(params["dbname"]);
    }
    (uri, params["table"])
}

fn gdal_ds_from_qgs_ds(ds: &str) -> (String, &str) {
    let parts = ds.split('|').collect::<Vec<&str>>();
    (parts[0].to_string(), parts[1])
}

#[test]
fn test_parse_xml() {
    assert!(read_xml("../examples/natural_earth.qgs").is_ok());
    assert_eq!(read_xml("wrong_file_name").err().unwrap().description(),
               "entity not found");
    assert_eq!(read_xml("Cargo.toml").err().unwrap().description(),
               "Malformed XML");
}

#[test]
fn test_pg_uri() {
    assert_eq!(pg_uri_from_qgs_ds("dbname=\'natural_earth_vectors\' host=localhost port=5432 user='trex' password='12345' sslmode=prefer key=\'tid\' estimatedmetadata=true srid=3857 type=Polygon table=\"public\".\"admin_0_countries\" (wkb_geometry) sql="),
               ("postgresql://trex:12345@localhost:5432/natural_earth_vectors".to_string(),
                r#""public"."admin_0_countries""#));
    assert_eq!(pg_uri_from_qgs_ds("dbname=\'natural_earth_vectors\' port=5432 sslmode=disable key=\'tid\' estimatedmetadata=true srid=3857 type=Polygon table=\"public\".\"admin_0_countries\" (wkb_geometry) sql="),
               ("postgresql://%2Frun%2Fpostgresql:5432/natural_earth_vectors".to_string(),
                r#""public"."admin_0_countries""#));
    assert_eq!(pg_uri_from_qgs_ds(r#"dbname='natural_earth_vectors' port=5432 sslmode=disable key='fid' estimatedmetadata=true srid=4326 type=Point table="public"."ne_10m_populated_places_wgs84" (wkb_geometry) sql="scalerank" &lt; 9"#),
               ("postgresql://%2Frun%2Fpostgresql:5432/natural_earth_vectors".to_string(), r#""public"."ne_10m_populated_places_wgs84""#));

}

#[test]
fn test_gdal_ds() {
    assert_eq!(gdal_ds_from_qgs_ds("../t-rex-gdal/natural_earth.gpkg|layerid=2"),
               ("../t-rex-gdal/natural_earth.gpkg".to_string(), "ne_110m_admin_0_countries"));
}

#[test]
fn read_qgs() {
    let root = read_xml("../examples/natural_earth.qgs").unwrap();
    let mut layers = Vec::new();
    let projectlayers = root.find("projectlayers")
        .expect("Invalid or empty QGIS Project file");
    for layer in projectlayers.find_all("maplayer") {
        let name = layer
            .find("layername")
            .expect("Missing element 'layername'")
            .text();
        let provider = layer
            .find("provider")
            .expect("Missing element 'provider'")
            .text();
        let dsinfo = layer
            .find("datasource")
            .expect("Missing element 'datasource'")
            .text();
        let (ds, table) = match provider {
            "ogr" => gdal_ds_from_qgs_ds(dsinfo),
            "postgres" => pg_uri_from_qgs_ds(dsinfo),
            _ => ("".to_string(), ""),
        };
        layers.push((name, ds, table))
    }
    assert_eq!(layers.len(), 4);
    assert_eq!(layers[0],
               ("admin_0_countries",
                "postgresql://%2Frun%2Fpostgresql:5432/natural_earth_vectors".to_string(),
                r#""public"."admin_0_countries""#));
    assert_eq!(layers[1],
               ("natural_earth ne_110m_admin_0_countries",
                "../t-rex-gdal/natural_earth.gpkg".to_string(),
                "ne_110m_admin_0_countries"));
}
