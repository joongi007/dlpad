# S10: watch500 기본 제품의 대형 진단 회귀

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R10, R13
- 담당/수정 범위: root — 기존 native runner 실행·새 evidence·본 문서; metrics_recovery — /tmp 직렬 helper만
- 선행 작업: 현재 source543 전체 검증, 기본 제품 SHA 복원 및 GUI CPU 측정 종료

## 사용자 결과

최초 릴리스 범위 S10의 대형 JSON/Markdown/미리보기 진단을 현재 기본 제품에서 열고 저장한다.

## 계약과 범위

CPU SHA59448518d46ae8f3da4bf18540ae8ed031497584f13799e231966dd28f754848·현재 debug fixture tool provenance·원 collector/source를 고정한다. fresh 합성 fixture와 private desktop을 사용한다. native 빌드/시험은 직렬 실행하며 사용자 계정·영상·Documents·clipboard를 사용하지 않는다. Cargo/schema 변경은 없다.

## 인수 조건

large-json/large-markdown/large-preview 세 case의 export byte/SHA·상한·privacy, 실제 Failed Job/Attempt·artifact 없음·DB 무결성·원 cookie/sentinel/calls, 미리보기 first/next/last 실제 PNG와 paging, 자연 app exit0/Job0·private/input과 소스/핀 불변을 root가 대조한다. 수집 성공만으로 완료하지 않으며 이전 3f9 snapshot 인수를 재사용하지 않는다.

## 검증과 handoff

하위가 /tmp 직렬 helper와 provenance 검토를 준비했다. GUI CPU 종료 후 현재 fixture tool build provenance부터 확인하고 실행한다. 아직 native 미실행이다.

현재 source543 Windows fixture tool 빌드가 통과했고 SHA256 a56c7990656c3121028b255cdbd16cc979fe9a7740699a0a30e9ca38310c9380를 기록했다. 기본 제품 SHA는 변하지 않았다. [빌드 provenance](../validation/evidence/windows-watch500-s10-fixture-build-20261006-continue/result.json). 세 case 순차 native 실행을 시작했다.

세 case 수집은 모두 성공했다. root 이미지 검토에서 최초 preview-first.png는 펼친 preview가 viewport 아래에 있어 실제 글자가 보이지 않았다. JSON text만으로 첫 페이지 시각 인수를 통과시키지 않고, 원 collector/pins와 실패 범위를 보존한 뒤 private 원 page control을 화면에 드러내고 읽기 내용/페이지1 불변을 확인하는 관측 보강으로 preview만 새 evidence에서 재실행한다. 제품 동작은 변경하지 않는다.

root가 JSON 1,841,855B/Markdown 1,933,102B export 원 bytes/SHA·redaction·logs128×4096B/MPD8×256nodes·Failed Job/Attempt 각1/artifact0·integrity·cookie/sentinel/calls·owned file inventory를 대조했다. 보강 preview의 first/next/last 236쪽 실제 PNG와 마지막 Next 비활성을 확인했다. [최종 scoped 대조](../validation/evidence/windows-watch500-s10-large-suite-20261006-continue/root-review.json). 원 collector를 suite original-collector.cs로 SHA 고정 보존했다. 최초 first PNG 미달은 기록 그대로 남으며 새 preview 증거만 시각 인수를 대체한다. exact2MiB/peak memory/clipboard/physical reader·IME는 이번 범위 미검증이다.
