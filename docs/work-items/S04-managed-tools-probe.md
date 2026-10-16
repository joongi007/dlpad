# S04: 격리 관리 설치 GUI 검증 harness

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R05, R08
- 담당/수정 범위: diagnostic_cache agent; scripts/managed_tools_probe.py, 이 작업 계약, docs/validation/managed-tools-gui-validation.md만
- 선행 작업: native 관리 UI와 최종 Linux binary(주 담당 제공)

## 사용자 결과

Xvfb의 실제 native Install/Cancel/UseActive 동작을 임시 app-data와 synthetic-media로 검증하며 사용자 데스크톱을 방해하지 않는다.

## 계약과 범위

README→PRODUCT→CONTEXT→R05/R08→ADR0003와 INTEGRATIONS를 읽었다. native_probe의 isolated display/screenshot, x11_diagnostics click helper만 재사용한다. visible fallback, Rust/Cargo/build/native_probe 수정, 실제 계정·영상·외부 설치 변경을 금지한다. source는 제품의 공식 고정 HTTPS pin만 사용한다. 성공은 active DB/manifest/executable SHA와 다운로드 입력 clipboard를 대조하며 screenshots를 남긴다. 취소는 staging 관측 뒤 클릭하고 activation 없음/소유 staging 종료를 관측한다. 실패는 소유 temp root의 foreign fixture를 보존하는 안전한 ownership 오류를 사용한다.

## 인수 조건

인자 오류는 프로세스/Xvfb 실행 전에 거절한다. 임시 sibling data/media, process group cleanup, timeout, JSON 오류 보고와 screenshots를 구현한다. 실제 binary 준비 전 arg 검증만 보고하고 GUI 성공을 주장하지 않는다.

## 검증과 handoff

주 담당이 제공한 Linux release binary SHA256 `f0aa2e335ce182f76290d71f41b9f7f8ffb965875c25b39b90d4965e93bdf842`를 사용했다. 빌드는 수행하지 않았다. `DLPAD_TEST_FONT=/mnt/c/Windows/Fonts/malgun.ttf python3 scripts/managed_tools_probe.py target/release/dlpad /tmp/dlpad-managed-gui-install-v6.json --install-point 216,272 --cancel-point 350,272 --use-point 150,444 --tool-point 250,350 --timeout 30` 실제 install/activepin/hash/source/UseActive downloads clipboard 대조 통과. `... /tmp/dlpad-managed-gui-cancel-v2.json --mode cancel --timeout 30` 실제 stage→중단→stage 정리/noactive 통과(4.417초). `... /tmp/dlpad-managed-gui-failure-v3.json --mode failure --install-point 216,272 --timeout 30` noactive/foreign bytes 보존 통과; screenshot 직접 검토에서 오류 문구 표시 확인. failure JSON은 자동 error text 검증이 없어 native_gui_verified=false를 유지한다.

AST와 인자8건 재통과. [실제 증거와 실패 이력](../validation/managed-tools-gui-validation.md)을 기록했다. Python3.10 file_digest 부재는64KiB streaming SHA로 수정했고 한글 폰트에서 실제 줄바꿈/버튼 좌표를 조정했다. WM 없는 Xvfb의 소유 창에만 focus를 지정하고 XKeysymToKeycode unsigned-char ABI를 정확히 선언했다. fixture font는 명시 --font/DLPAD_TEST_FONT로 요구한다. 사용자 desktop/실제 영상/외부 설치 사용 없음.

단일 공식 pin이므로 보관 버전 선택/제거 GUI, 기존 active 보존을 포함한 GUI update 실패/취소, other OS/IME/screen reader는 미검증이다. 이후 앱 변경을 이 binary의 GUI 통과로 취급하지 않는다. 주 담당은 보존 JSON/screenshots를 독립 검토한다.
