# S05: CPU Tab 탐색 native 인수

- 상태: completed (Linux CPU 다섯 도움말 영역 탐색)
- 분류: validation
- 우선순위: P0
- 요구사항: R07, R01
- 담당/수정 범위: soak_recovery_audit — scripts/cpu_tab_navigation_probe.py, 이 문서
- 선행 작업: immutable CPU ca8e969f028fdd50f5c7f9ae19c3c71928d7740daa5c93189ec100ee9f5fc77e, 기존 owned Xvfb/AT-SPI helpers

## 사용자 결과

영상 URL부터 다섯 옵션 도움말까지 실제 Tab/ShiftTab 탐색 가능 여부를 별도로 관측한다.

## 계약과 범위

private Xvfb·private bus·합성 임시 데이터만 사용한다. 초기 URL은 소유 좌표로 클릭하고 이후 GrabFocus 없이 Tab/ShiftTab만 사용한다. 각 키의 실제 focused name/role/state/bounds와 private PNG를 남긴다. 다른 scripts/Rust/Cargo 및 사용자 desktop/clipboard를 변경하지 않는다.

## 인수 조건

최대 64 forward Tab으로 다섯 도움말을 모두 거친 뒤 같은 수의 ShiftTab으로 실제 URL에 복귀한다. 접근성 조회 실패·복수 포커스·미관측·화면 밖 포커스는 실패한다. job/tool 미실행 및 소유 자원 회수를 검증한다. 시각적 포커스 표시의 충분성은 원시 screenshot 독립 검토가 필요하다.

## 검증과 handoff

Python `compile(..., 'exec')` 실제 exit 0. root의 실제 private GUI 첫 실행도 exit 0이며 [원시 결과](../validation/evidence/p0-cpu-tab-navigation-native-first/result.json)는 `passed`와 `tab_navigation_verified`가 true이다. binary의 전후 SHA256은 위 immutable 값과 동일하고, 수집기 동결 SHA256은 `04745b69b929aaac79dd581289a9517934a65ed5188a154a0c590d56249a87d3`이다.

실제 forward Tab 16회에서 FFmpeg(4), 쿠키(6), 품질(10), 구간(13), 자르기(16) 도움말에 도달했다. ShiftTab 16회 뒤 영상 URL에 복귀했다. 초기 관측을 포함한 33개 실제 focus 관측과 private PNG를 보존했다. job/tool 미실행, owned cleanup, desktop fallback 미사용도 원시 결과로 확인했다. 기존 원시 파일과 수집기는 수정하지 않았다.

root가 다섯 header PNG를 독립 확인했으며 현재 title/arrow의 흰색과 주변 회색 차이로 focused header를 식별할 수 있었다. 다만 이전 field outline도 남아 있어 시각적 포커스 명확성의 최종 접근성 판정은 남는다. `visual_focus_review`는 `REVIEW_REQUIRED`를 유지한다.

인수 범위는 Linux CPU 옵션의 URL부터 다섯 도움말 영역까지의 실제 Tab/ShiftTab 탐색이다. 전체 앱의 시작 버튼·다른 페이지, 기본 Iced UI, screen reader, Windows/macOS 또는 3 OS 전체 접근성 완료로 확대하지 않는다. 256-node traversal의 호출별 timeout 때문에 전체 탐색 관측 상한은 180초이며 최대 128개 키이다. 개별 외부 호출 때문에 엄격한 wall-clock 상한과는 다르므로 재실행에는 별도 root timeout이 필요하다.
