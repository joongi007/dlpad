# S00: 확대 후 상태 표시줄 재배치와 배율 조작 공개

- 상태: completed (scoped)
- 분류: UX
- 우선순위: P0
- 요구사항: R06, R08
- 담당/수정 범위: root — 소스 채택·actual 실행·통합 검증·본 기록; advanced_units — /tmp accessible_settings.rs 및 accessible.rs 실제 Candidate 회귀 후보만
- 선행 작업: v13 whole1,297/0/30와 최신 native 확대 진단·원소스 동결 감사 종료

## 사용자 결과

배율을 바꾼 뒤 상태 표시줄이 늦게 줄바꿈되어도 선택한 배율 조작의 아래쪽을 확인할 수 있다. 사용자가 직접 스크롤한 뒤에는 자동으로 되돌리지 않는다.

## 계약과 범위

Rust native와 공통 token/원 ID/선택 값/worker/DB/schema/Cargo는 유지한다. v13 native 첫 진단의 scale200-after-250.png에서 footer 두 줄과 배율 상자 아래 잘림을 root가 직접 확인했다. 이후 표본·현재63/warmup은 전체 조작이 보인다. 기존 8frame 안정 viewport 공개를 소비한 뒤 viewport가 늦게 줄어드는 인과는 아직 가설이며 실제 Candidate 구성 회귀로 확인한다. observer IsOffscreen 미지원이나 앞선 unfocused Theme 라벨 숨김을 오류로 집계하지 않는다.

## 인수 조건

실제 Candidate에서 배율 선택→초기 공개 안정화→production footer 줄바꿈에 의한 viewport 축소의 RED/GREEN을 확인한다. 행이 들어가면 라벨/설명/조작을, 행이 너무 크면 전체 조작을 공개한다. 같은 frame multipass·사용자 wheel/press 중단·불필요 반복 repaint·selection/ID 보존을 검증한다. 채택 시 fmt/clippy/test/release와 원로그·최신 native 화면을 별도 확인하고 설치 후보를 갱신한다. 물리 포인터/IME/reader/OS DPI는 별도이다.

## 검증과 handoff

native 원표본과 FAIL은 보존한다. 실제 회귀/수정은 /tmp 후보 준비 단계이며 성공을 가정하지 않는다. 기존 메뉴 두 개 관찰은 원63의 정확한 두 이미지를 root가 다시 열어 다섯 메뉴를 확인했으므로 오관찰 정정 기록으로 분리한다.

첫 test-only baseline의 실제 Windows 필터 실행은Rust101/0성공1실패였다. 그러나 초기/후기 footer 높이가28로 같아 ‘production footer adds a line’ 조건에서 실패한 시험 fixture 문제이며 제품 RED가 아니다. source/GNU/lock/자연 Job0와 원로그를 root가 확인해 보존했고 조건을 교정한 새 fixture를 준비 중이다. [원 조건 실패](../validation/evidence/windows-msvc-late-footer-scale-baseline-red-20261013/root-failure-classification.json). 현재 제품 동작은 v13과 같으며 새 테스트가 추가된 개발 소스와 배포 후보의 검증 범위를 구분한다.

두 번째 원동작 실험은 실제 footer28→48·body clip bottom271→251인데 Scale bottom263이 유지되어12logical px 잘림을 fullcontext 인수에서 재현했다. [실제 RED](../validation/evidence/windows-msvc-late-footer-scale-baseline-red-v2-20261013/root-failure-review.json). 최소 수정은 Scale 선택 이후 최초 늦은 수직 viewport 축소를 한 번만 기존8frame 안정 공개로 보정한다. 타이머·heap·I/O·dependency 추가가 없고 사용자 wheel/press, 페이지 이동, 새 키/문자/IME 입력으로 watch를 해제한다. root는 실제 Escape 취소 회귀도 추가했다.

필터 GREEN 두 회는 지연 공개/키보드 취소/페이지 복귀/기존 실제 footer4개가 성공했고 wheel observer의 ‘background painted’ 조건1개가 실패했다. 원96입력과12입력 모두 기록을 보존한다. 완전히 offscroll된 paint의 정상 생략인지 stale 접근성 cache인지 출력 geometry로 진단하며 guard를 제거해 성공으로 바꾸지 않는다. [첫 원GREEN](../validation/evidence/windows-msvc-late-footer-scale-green-20261013/result.json), [두 번째 원GREEN](../validation/evidence/windows-msvc-late-footer-scale-green-v2-20261013/result.json). 전체 인수·최신 실행 파일 갱신은 아직 미완료다.

진단에서 실패한 pointer 입력의 실좌표448이 scrollbar track448..456에 있었고, 현재/rooted/update Scale bound280.5..312.5가 body bottom271 밖으로 이동했음을 확인했다. stale cache는 아니며 정상적인 offscroll paint 생략이었다. 원진단/source를 보존한 뒤 빈 본문 x404로만 입력 좌표를 수정하고 진단 코드는 테스트 본문에서 제거했다. 휠12 입력은 실제 이동>1, 일부paint/intersection, footer높이증가, 위치보존/noSnapback 검사를 유지한다. 최종 필터5개는 실제 전부 성공했다. [원진단](../validation/evidence/windows-msvc-late-footer-wheel-snapshot-diagnostic-20261013/root-diagnostic-review.json), [5개 GREEN](../validation/evidence/windows-msvc-late-footer-scale-green-v3-20261013/root-review.json). Root 전체 검사·새 패키지·native 검수는 진행 중이다.


## 현재 통합 결과

최종 소스 Windows fmt·strict clippy·전체test·release는1,305/0/30·43harness이며 원로그/current656/3PE/GNU/lock을 root가 독립 대조했다. 앱은9,838,080B로1,536B 증가했다. 설치기·CLI·schema·Cargo·dependency 변화 없음. 추가 타이머/heap/I/O 없이 첫 늦은 수직축소 한 번만 보정한다. 계속되는 모든 viewport 변경 보장이나 물리IME 입력 인수는 아니다.

현재 앱63개 PNG는 팀 직접 열람(하위28+28/root7), exactSHA·namespace·현재source·3DB·자연0/Job0를 대조했다. 150/200 활성배율 라벨/설명/전체콤보와 다섯 메뉴를 확인했다. 설치ZIP16,727,221B는 currentproof/3PE/archiveCRC·선택noticegraph를 대조했다. 별도8quiet표본/10PNG native자연0/Job0 및 최신설치 변조 거부·독립27ID도 확인했다. 전체GUI/physicalhover/focus/IME/reader/OSDPI·legal/signing은 별도이다.

[현재 whole](../validation/evidence/windows-msvc-scale-viewport-reflow-whole-20261013/root-review.json), [63개 화면](../validation/evidence/current-scale-viewport-reflow-app63-20261013/root-team-visual-review.json), [패키지](../validation/evidence/windows-msvc-ui-cross-surface-v14-20261013-package-root-review.json). 아래 완료 이전 문장은 당시 실험 이력이며 원FAIL/fixture실패/observer실패를 성공으로 덮어쓰지 않는다.
