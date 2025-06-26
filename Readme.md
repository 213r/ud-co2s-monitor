# udco2s測定モニター

このプロジェクトは、CO2センサーからデータを取得し、InfluxDBなどの時系列データベースにエクスポートするRust製のモニタリングツールです。

## 特徴

- CO2センサーからのデータ収集
- InfluxDBへのデータエクスポート
- Grafanaによる可視化用ダッシュボード

## 設定
`config.yaml` に各種設定を記述します。`config_example.yaml`を参考にしてください。


## 
```
docker compose up -d # grafana, influxdbの起動 
cross run --release  # センサーデータの取得、InfluxDBへのエキスポート
```

## crossコンパイ (raspi向け)
```
cargo install cross
cross build --release --target aarch64-unknown-linux-musl
```