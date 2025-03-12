vendor/galileo:
	mkdir -p vendor/galileo
	git clone git@github.com:Maximkaaa/galileo.git vendor/galileo
	patch --strip=1 --directory=vendor/galileo < patches/galileo.patch

vendor: vendor/galileo

.PHONY: vendor
