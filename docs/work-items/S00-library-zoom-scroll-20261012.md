# S00: 확대 파일 목록의 전체 내용 접근

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: 주 담당 통합; release_preparation은 /tmp 후보만. crates/app/src/accessible_library.rs, 이 문서
- 선행 작업: S00-current-five-screen-ui-polish-20261011

## 사용자 결과

800×600의 200% 화면에서도 저장 폴더·목록 동작·빈 안내·파일 행을 스크롤로 확인한다.

## 계약과 범위

주 담당이 실제 200-library-after PNG와 LibraryPanel.show를 대조했다. 안내와 도구막대는 기존 rows ScrollArea 밖에 있고 빈 rows는 scroll 높이가 0이라 숨은 안내를 공개할 경로가 없다. 단일 전체 내용 scroll을 사용하며 파일 행은 보이는 범위만 렌더링한다. 기존 PAGE128·scan transaction·선택·picker·논리 ID를 유지한다. 중첩 scroll과 전체 목록 무제한 렌더링을 도입하지 않는다.

## 인수 조건

실제 LibraryPanel.show와 Context에서 빈 안내 접근 및 200% scroll을 재현한다. 보이는 행 범위와 ID·선택을 검사하고 읽기 실패에서 이전 목록 보존을 유지한다. 최신 Windows 가상 화면에서 빈 목록과 파일이 있는 목록을 확인한다. 현재 빈 화면 근거를 실제 populated 목록·물리 Tab 성공으로 확대하지 않는다.

## 검증과 handoff

독립 읽기 감사는 /tmp/dlpad-library-200-viewport-audit에 있다. 수정 후보·테스트는 /tmp에서 준비하며 프로젝트 통합·native 실행은 주 담당이 수행한다. 설정·지표 작업과 담당 파일이 겹치지 않는다.

### Windows RED 확인 및 후보 통합

`windows-msvc-library-zoom-scroll-red-20261012/test.log`에서 기존 구현의 실제 wheel 회귀가 실패했다. 컴파일 성공; 소유 Job natural-zero=True, forced=False, input desktop unchanged. 전체 페이지 단일 ScrollArea와 화면에 필요한 행만 생성하는 후보를 주 담당이 검토하여 통합했다. GREEN 및 native 화면 검증은 아직 미완료다.

### 첫 통합 검증 결과

`windows-msvc-scale-footer-library-ui-whole-20261012`: fmt 및 strict clippy 통과. workspace test는 첫 UI harness 228 passed / 2 failed / 3 ignored에서 중단되어 release build는 실행되지 않았다. 배율 선택 회귀와 기존 테마 문맥 회귀는 통과했다. footer 좁은48px wrap 및 library wheel 안내 도달 회귀는 실패하여 원인 조사 중이다. 전체 통과 또는 UI 완료로 주장하지 않는다. 소유 Job 자연 종료/정리 및 input desktop unchanged 확인.

### 추가 원인 계측

기존 wheel 이벤트와 인수 조건을 유지하고 cfg(test)에서만 실제 offset/content/inner rect 및 안내 text/clip containment를 출력하도록 계측했다. 제품 런타임 로깅은 추가하지 않았다. `windows-msvc-footer-v2-library-scroll-diagnosis-20261012` 실행 중이며 원인은 아직 확정하지 않았다.

### wheel 계측 결과와 시험 교정

실제 offset 0→162→311(max), content 457 및 viewport 높이146을 확인했다. 마지막 위치의 안내 top88/clip top89로 1px 잘렸으므로 기존 complete-clip assertion은 올바르게 실패했다. 큰180px 이동이 도달 가능한 중간 위치를 건너뛰는 시험 문제를 확인하여 안정화 뒤24px 실제 wheel을 최대32회 진행한다. 마지막 화면에 안내 전체가 온전히 보이는 조건을 유지하고 임시 계측은 제거했다. 같은 수정 시험으로 기존 생산 코드 RED를 다시 확인할 예정이다. 전체 v2 검증 실행 중.

### v2 workspace test 종료 확인

원본 `windows-msvc-scale-footer-library-ui-whole-v2-20261012/test.log`를 주 담당이 집계하여43 harness / 1,111 passed / 0 failed / 28 ignored를 확인했다. 테스트 Job natural-zero/forced=False/input desktop unchanged. release build는 아직 진행 중이므로 전체 검증 완료로 기록하지 않는다. 교정된 footer/library RED와 GREEN 테스트 본문은 rustfmt 후 바이트 동일성을 독립 확인했다.

### v2 전체 검증 완료

`windows-msvc-scale-footer-library-ui-whole-v2-20261012/root-review.json`에서43 harness / 1,111 passed / 0 failed / 28 ignored, fmt·strict clippy·locked CPU workspace test·release build의 실제4명령 및 current source/hash/artifact를 주 담당이 독립 확인했다. CPU 9,670,144 bytes SHA256 `252b7bbaf3a476e574af5925ca392c3680f891a8f5c6b25024d40be88562c92f`; Cargo workspace/lock/schema 변경 없음. `current-ui-scale-selected-context-layout-20261012` 가상 화면 실행 중. 사용자 시각 인수·설치본 갱신은 미완료다.

### 교정된 동일 시험의 독립 RED/GREEN

`windows-msvc-scale-footer-library-corrected-original-red-20261012/test.log`: 원 생산 코드 + 동일 교정 시험에서141 passed / 2 failed. footer는 실제 logical400/row384/natural106.84375/clip400×300에서 문구 분리, library는24px incremental wheel에서도 안내 전체 도달 실패. 이전 whole-v2 동일 시험 GREEN1111/0와 비교하여 수정 효과를 확인했다. 원본 파일 SHA 및 포맷 후 테스트 본문 byte 동일성을 확인했으며 RED 종료 후 저장한 GREEN bytes를 정확히 복원했다. Job natural-zero/forced=False/input desktop unchanged.

### 합성128 native 관측 v3 실패

`windows-library200-empty-populated128-private-v3-20261012` C# compile는 성공했다. 앱 GUI는 `Actual wheel empty instruction fully reachable`에서 실패해128개 fixture 생성/scan/selection에는 도달하지 못했다. 성공으로 기록하지 않는다. 소유 가상 화면의 WM wheel 전달과 실제 viewport 좌표 변화를 조사 중이다. 이전 Context incremental wheel 성공을 native 입력 성공으로 대체하지 않는다.

### native wheel 전달 범위

128 v3 관측의32개 body geometry에서 target y674..744/body195..432가 동일했다. WM wheel 전달 후 실제 이동은 관찰되지 않았으며 이를 제품 스크롤 실패로 단정하지 않는다. 기존 Context wheel 성공과 별도로 native 입력 미검증을 유지한다. 실제 광고되는 UIA scroll pattern만 확인하는 후보를 준비했으나 미지원 대체 동작을 가정하거나 제품 CursorLeft를 무시하지 않는다.
