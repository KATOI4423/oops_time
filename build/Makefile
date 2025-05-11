######################
# Makefile for cargo #
######################

.PHONY: all
all: license windows

.PHONY: clean
clean: linux-clean windows-clean

.PHONY: dist-clean
dist-clean:
	rm -rf ${BUILD_DIR}/*

.PHONY: linux
linux:
	@cd ${TOP_DIR} \
	&& cargo tauri build -- --target-dir ${BUILD_DIR}

.PHONY: linux-clean
linux-clean:
	rm -rf ${BUILD_LINUX_DIR}

.PHONY: windows
windows:
	@cd ${TOP_DIR} \
	&& . ${BUILD_DIR}/windows_env.sh \
	&& cargo tauri build --target x86_64-pc-windows-gnu -- --target-dir ${BUILD_DIR}

.PHONY: windows-clean
windows-clean:
	rm -rf ${BUILD_WINDOWS_DIR}

REGFILE := "${BUILD_REG_DIR}/register.reg"
UNREGFILE := "${BUILD_REG_DIR}/unregister.reg"

.PHONY: reg
reg:
	@mkdir -p ${BUILD_REG_DIR}
	@./mkreg.sh ${REGFILE} ${UNREGFILE}

.PHONY: reg-clean
reg-clean:
	@-rm ${REGFILE} ${UNREGFILE}

.PHONY: license
license:
	@cd ${TAURI_DIR} \
	&& cargo about generate ${TAURI_DIR}/about.hbs > ${TOP_DIR}/License.html
	python3 ${BUILD_DIR}/merge_licenses.py ${TOP_DIR}/License.html ${TAURI_DIR}/about.hbs ${TOP_DIR}/License.html
