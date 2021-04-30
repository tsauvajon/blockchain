cover:
	cargo tarpaulin --out Html
	miniserve tarpaulin-report.html
