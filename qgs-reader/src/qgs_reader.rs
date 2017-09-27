//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use elementtree::Element;
use std::fs::File;
use std::io::BufReader;

#[test]
fn read_qgs() {
    let file = File::open("../examples/natural_earth.qgs").unwrap();
    let mut reader = BufReader::new(file);
    let root = Element::from_reader(&mut reader).unwrap();

    let mut layers = Vec::new();
    let projectlayers = root.find("projectlayers").expect("Invalid or empty QGIS Project file");
    for layer in projectlayers.find_all("maplayer") {
        let name = layer.find("layername").expect("Missing element 'layername'").text();
        let provider = layer.find("provider").expect("Missing element 'provider'").text();
        let dsinfo = layer.find("datasource").expect("Missing element 'datasource'").text();
        layers.push((name, provider, dsinfo))
    }
    assert_eq!(layers.len(), 4);
    assert_eq!(layers[0], ("admin_0_countries", "postgres", "dbname=\'natural_earth_vectors\' port=5432 sslmode=disable key=\'tid\' estimatedmetadata=true srid=3857 type=Polygon table=\"public\".\"admin_0_countries\" (wkb_geometry) sql="));
    assert_eq!(layers[1], ("natural_earth ne_110m_admin_0_countries", "ogr", "../t-rex-gdal/natural_earth.gpkg|layerid=2"));
}
