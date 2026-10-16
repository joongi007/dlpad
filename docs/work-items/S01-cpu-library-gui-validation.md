# S01: CPU 파일 목록 실제 GUI 검증

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R03, R08, S01
- 담당/수정 범위: diagnostic_cache; 신규 scripts/cpu_library_probe.py, docs/validation/cpu-library-gui-validation.md 및 cpu-library-gui evidence만
- 선행 작업: root CPU library integration/단일 panel worker/협조 scan 취소, READY SHA185442b2dd209aa8e2ce43251c5ef695c835fe1c60c33c4de3bfb0d15d7971ad 및20 tests/fmt/clippy/release 통과

## 사용자 결과

실제 CPU 파일 목록 탭에서 합성10,000파일·128개 페이지·Next·외부 변경과 refresh·루트 실패보존·닫기 대기·keyboard label을 확인한다.

## 계약과 범위

제품/source/Cargo/공유 probe 변경 금지. 기존 docs/work-items/S01-cpu-library-{panel,integration} 계약을 따른다. owned Xvfb/private bus/temp XDG/app-data/media siblings만 사용한다. 외부추가/rename/delete는 이번 소유 합성 fixture만 변경한다. 사용자 desktop/실제 미디어/Documents/clipboard/audio 금지. 실제 newhash에 한정하고 이전 native 증거를 확대하지 않는다.

## 인수 조건

실제 PNG/AT-SPI/SQLite snapshot·10,000관측·첫/두번째128페이지 시작끝·watch/refresh의수렴·invalidroot이전목록/다운로드루트보존을 확인한다. SQLite 소유 fixture lock으로 scan 진행을 제한하여 CloseWait 응답/종료를 관측하고 원 worker signal을 직접 관측하지 못하면 범위를 명시한다. elapsed/RSS/CPU와 모든 소유 직접 child 회수·temp제거·끝 SHA를 기록한다. 원 실패는 보존한다.

## 검증과 handoff

실제 세 차례 실행과 raw 보존 완료. 목록/페이지/외부 변경/실패 루트/Tab 확인. CloseWait는 busy 진입을 입증하지 못하여 미충족이며 제품 성공으로 보고하지 않는다. 전용 검증 문서: docs/validation/cpu-library-gui-validation.md. 소유 GUI 종료 완료, root에 source freeze 해제 통보.
