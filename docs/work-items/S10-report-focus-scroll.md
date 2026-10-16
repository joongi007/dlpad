# S10: 보고서 포커스 가시화

- 상태: done
- 분류: fix
- 우선순위: P0
- 요구사항: R08, R13
- 담당/수정 범위: metrics_recovery — accessible.rs, accessible_diagnostics.rs, 이 문서; root — Cargo와 실제 private GUI 통합 검증
- 선행 작업: 정상960x640 privateGUI actualfocus/가시화 실패

## 사용자 결과

접근성·키보드 포커스로 보고서 header와 저장 경로를 선택하면 각 제어 요소가 해당 스크롤 영역으로 이동한다.

## 계약과 범위

gained_focus 시 scroll_to_me Center를 요청한다. 사용자 desktop·전역포커스는 변경하지 않는다. 중첩300높이 영역에서 모든 제어요소가 동시에 보여야 한다는 조건을 만들지 않고 순차 가시 접근을 실제검증한다.

## 인수 조건

정상960x640에서 header·export·preview 순차 접근 및 실제bounds/privatePNG. 기존Readonly/privacy·새파일저장·덮어쓰기거절 유지. 실제OS검증전완료표시금지.

## 검증과 handoff

root 구현 후 새immutable Windows build/GUI 검증 예정.

readonlypreview node는 egui가 이미 처리하는 ScrollIntoView action을 명시 광고한다. Focus 불가/ReadOnly flag는 유지한다. nested300 영역의 header/preview/export는 순차 실제 가시검증하며 동시가시를요구하지않는다. 제품ScrollIntoView 실제 지원은 새Windows native시험으로확인한다.

## 중첩 스크롤 수정 범위

preview의ScrollIntoView는가장안쪽ScrollArea에서target을소비하고exportfocus는diagnostic-controls에서소비돼download-content까지전달되지않는다. 실제AccessKitbounds는ancestorclip을반영하지않아nodebounds/text성공이paint가시증거가아니다. 이수정은widget생성전exactID의ScrollIntoView를확인하고preview영역의target을controls와downloadancestor에각각다시전달한다. exportgained_focus와exactScrollIntoView도동일하게전달한다. Focus/readonly/stringprivacy/Cargo는변경하지않는다.

## 제품 수정 동결 handoff

DiagnosticPanel::show는 Option<egui::Rect>를 반환한다. 저장 경로와 readonly preview에 explicit persistent ID를 지정하고, widget 생성 전에 InputState::has_accesskit_action_request(id, Action::ScrollIntoView)로 정확한 요청을 확인한다. egui가 widget 생성 중 action을 소비하므로 생성 후 조회하지 않는다. 저장 경로는 gained_focus 또는 exact ScrollIntoView 때 response target을 유지하고 ancestor 요청을 표시한다. readonly preview는 Focus 동작을 추가하지 않고 정확 ScrollIntoView만 처리한다.

preview ScrollArea가 기존 TextEdit target을 소비한 뒤 bounded preview viewport를 diagnostic-controls에 다시 scroll_to_rect한다. controls가 그 요청을 소비한 뒤 accessible.rs caller가 controls viewport를 download-content에 다시 scroll_to_rect한다. 큰 report 전체 TextEdit rect를 outer target으로 사용하지 않아 bounded 영역을 드러내며, 세 영역의 offset은 각각 기존 egui 정책으로 조정한다. header focus의 기존 바깥 요청은 유지한다. UI strings, catalog, readonly, privacy, report pagination과 제품 웹서버 여부는 바뀌지 않는다.

actual egui RawInput AccessKit action frame을 사용하는 신규 unit readonly_preview_scroll_action_returns_ancestor_target_only_for_exact_request를 작성했다. unrelated node ScrollIntoView와 readonly Focus는 ancestor target이 없고, exact preview ScrollIntoView는 유한320 이하 viewport target을 반환해야 한다. show의 호출부는 반환 Option을 controls.inner 또는 테스트 target에 사용하며 unused must-use로 버리지 않는다. 시험은 아직 실행하지 않았고 root Cargo와 실제 private GUI를 기다린다.

accessible.rs SHA256 `7b908393410ba72df61fb283368ba62590755f992ad161a0bf6a5be3d84b83cd`; accessible_diagnostics.rs SHA256 `552693a542ddcd95bf45ca90ae7483d188c485f29346f11043687189fc63bf2e`. rustfmt 두 파일 exit0, Cargo/build 미실행. 실제960x640 PNG에서 preview body와 export의 viewport intersection 및 외부 offset 변화/순차 가시성을 root가 검증하기 전 S10 완료로 보고하지 않는다. node bounds/Text.GetText/action exit0만으로 실제 paint PASS를 주장하지 않는다.

## root 실제 통합 검증

CPU app 실제16+70+75시험 PASS(새exact action 회귀가두bin에서통과), workspace CPU all-targetclippy exit0. 새 Windows immutable SHA af89d78fbce7fa2cec188657635845e6a444bd0ce1d5d6dd59b32c378121cb3c releasebuild exit0/281입력SHA전후및복사시현재일치. normal960x640 privateGUI exit0에서header/export실제가시, previewPNG의본문 두 줄을root직접확인했다. Collector REVIEW_REQUIRED는수정하지않고 독립 root-review.json으로부분본문paint인수했다. readonly/preview-export1478Bhash동일/privacy/sentinel덮어쓰기거절/Failed1Attempt0artifact/copy회수/Job0/defaultdesktop유지확인. evidence windows-cookie-gui-auth-report-ancestor-first 및 report-ancestor-windows-release. Clipboard·실제IME·다른OS·전체보고서동시가시는완료주장이아니다.
