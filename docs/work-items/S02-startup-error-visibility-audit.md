# S02: 시작 복구 오류 안내의 실제 가시성 감사

- 상태: done
- 분류: review
- 우선순위: P0
- 요구사항: R08, R11, R13
- 담당/수정 범위: metrics_recovery — 이 문서만
- 선행 작업: Windows 시작 복구 error-first 사설 GUI 원시 증거

## 사용자 결과와 실제 증거

[원시 recovery.png](../validation/evidence/windows-cookie-startup-gui-error-first/gui/recovery.png)를 실제 열어 확인했다. 960×640 client PNG에서 오류 문자열은 약 y585 한 줄로 보이고, 그 위 약 y560에 '이전 작업 다시 확인' 버튼이 보인다. 따라서 이 파일은 오류 문자열이 viewport 밖이라는 제보를 재현하지 않는다. 아래 볼륨·수신 지표 두 줄도 보인다. result.json은 동일 960×640, start_enabled=false, input desktop unchanged를 기록한다. 다른 screenshot이나 DPI 조건의 비가시성까지 부정하는 결과는 아니다.

## 실제 코드와 최소 수정 후보

accessible.rs651의 ui.label(self.status)는 status TopBottomPanel에 있고 download-content ScrollArea 밖이다. retry 버튼은707의 download-actions 별도 bottom panel에 있다. 오류 발생시 ready=false인 상태에서 시작 버튼만 비활성이고 retry는 다른 worker/close_wait 조건이 없으면 활성이다. 따라서 현재 증거로 scroll_to_rect 또는 오류 label 이동을 적용할 근거가 없다. 하단 status는 ui.label 기본 wrapping에 의존하므로 좁은 폭·큰 글꼴에서는 실측 검증이 추가로 필요하다.

windows_cookie_startup_runner.cs는 HasLabel과 시작 버튼 enabled로 상태를 판정하며 오류 label/retry의 UIA bounds 및 client viewport 교차를 요구하지 않는다. UIA 존재가 실제 읽을 수 있는 화면을 뜻하지 않으므로 collector 인수 보강이 우선이다. 오류 label 및 retry 버튼의 IsOffscreen, BoundingRectangle, original HWND client좌표 교차영역과 PNG crop을 저장하고 label 전체 height와 retry 버튼 전체 영역이 footer 안에 있는지 확인해야 한다. 화면 밖이면 같은 source snapshot에서 재현한 뒤 최소 panel sizing/wrap 또는 persistent recovery banner를 제품 후보로 비교한다. UIA 자체 metadata를 PNG로 확대 주장하지 않는다.

## 키보드 및 실패 인수 조건

사설 desktop에서 원 입력 경로만 사용해 Tab/ShiftTab으로 retry까지 이동하고 실제 focus bounds가 보이는지 확인한다. Space/Enter로 retry를 한 번 실행해 recovering 고정 안내→동일 원 오류 안내 복귀 및 자동 다운로드0을 확인한다. source/cookieowner/sentinel 원 바이트를 보존한다. URL/cookie 필드는 비어 있고 start/resume는 준비 실패 동안 비활성이다. 원래 960×640 및 좁은 viewport/확대 글꼴 조건을 별도 측정한다. visible desktop fallback과 사용자 clipboard/전역 입력은 없다.

## 검증과 handoff

Rust/Cargo/vendor/runner/C# source를 수정하거나 GUI를 실행하지 않았다. 기존 PNG 시각 열람과 실제 코드 읽기만 수행했다. 이번 감사는 error-first에서 오류 안내·retry가 보인다는 관측이며 키보드 재시도나 다른 DPI 실제 검증 완료가 아니다. 후속 collector 가시성·키보드 인수는 root가 원시 증거를 추가로 생성해야 한다.

## 원본 경로·hash 대조

root가 canonical /mnt/d/Project/dlpad/docs/validation/evidence/windows-cookie-startup-gui-error-first/gui/recovery.png를 재검토하여 동일 오류 문자열 y585를 확인했다. 원본 SHA256은 c36b5e72384d66fadab6ae1589ec774e689136851845dfa84dfbf9b496cc68fb, 크기는68181바이트다. 초기 root lowercase 경로 표시본과 달랐던 관측은 canonical 경로 및 exact hash로 구분한다. 이 canonical 증거를 기준으로 현재960×640 가시성은 확인됐다. 제품 UI 수정은 필요하지 않다. runner는 UIA 상태 충족 직후 Capture하므로 후속 재현의 비가시성이 있다면 paint 안정화 지연 여부를 계측해야 한다. 현재 저장 tree/result에는 label별 bounds 인수값이 없어 metadata geometry 검증은 여전히 후속이다.
