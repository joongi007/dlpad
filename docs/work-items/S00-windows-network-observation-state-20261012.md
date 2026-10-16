# S00: Windows 네트워크 관측 설정 상태

- 상태: preparation
- 분류: ui-quality
- 우선순위: P0
- 요구사항: R01, R08
- 담당/수정 범위: 주 담당 catalog.rs/통합/native observer; diagnostic_cache accessible_metrics.rs 후보(/tmp만)
- 선행 작업: 실제Windows counter capability 감사(완료)

## 사용자 결과

Windows에서 작동하지 않는 Linux용 인터페이스 설정을 유효한 관측 설정처럼 제시하지 않는다. 미지원 이유와 실제 다운로드 속도 관측과의 차이를 설명한다. 저장된 설정값은 보존한다.

## 계약과 범위

R01의 unsupportedOS 측정불가 계약 유지. counter API/worker/schema/Cargo 변경 없음. Linux 지원 관측과 기존 유효성 검증은 유지한다. Windows GetIfEntry2 후보는 별도 후속이며 이번 구현으로 주장하지 않는다.

## 인수 조건

Windows interface/Apply disabled와 이유가 항상 보임; 저장값 변경 없음; Linux supported controls 기존 동작 유지; sourceunknown을0으로표시하지않음; 실제native800/200 label/help/disabled controls 가시성. native focus fixture는지원가능한입력과미지원설명상태를구분하며성공조건을혼동하지않는다.

## 검증과 handoff

Root sample_network_interface nonLinuxOkNone 직접확인. R01unsupported허용과대조해UX범위로결정. 후보 준비/실행검증전이다.

주 담당이 중앙catalog에비Linux미지원placeholder와NETWORK_COUNTER_UNAVAILABLE설명을추가했다. Linux예시유지. 다운로드속도와interface전체수신송신관측을구분하며저장설정보존을안내한다. accessible_metrics 지원상태candidate 통합 및compile/native검증은아직이다. Cargo/schema/backend추가없음.

주 담당candidate diff 검토와원본SHA대조후통합했다. nonLinuxinput/Apply disabled,중앙미지원이유visible,설정바이트보존; actualContext는disabledFocus/Text/Click에도edit/selection/revision불변을검증한다. Linuxeditable-invalidfocus회귀는Linuxcfg로보존,Windows부재capability를그회귀의성공으로대신하지않는다. 컴파일/native검증전.

Windowswhole rawtest.log현재43harness1119/0/28(terminalcollector/release검토는아직). 신규unsupportedFocus/Text/Click불변회귀통과. 기존Linuxeditable-invalidfocus시험은Windows비지원이므로실행되지않아총수가전과동일하다. nativeobserver후보는disabledSetFocus를제거하고input/Applydisabled·값보존·명시reason존재를요구,fullinputgeometry유지,reasonfullvisibility별도미검증. validator9tests주담당재실행PASS;C#native실행전이다.

주담당이새nativeobserverdiff를검토했고boundedstream cap+1로helper읽기를보완한freshcollector /tmp/dlpad-network-unavailable-native-final을준비했다. 원래필수63/확장48/배율5재사용,unsupported receipt는input/Applydisabled/값보존/reason존재를구분한다. 관련9tests재실행PASS. 현재release 검증완료전이라native아직미실행.

### 확대 화면 직접 관측: 안내 공개 미완료

주 담당이 실제 `200-settings-expand-after.png`를 열었다(SHA-256 `37acd493da6c48f21ae8db487a2927fe44ca6bf468b48354e6cc175a4c3a0458`). 확장 제목은 보이지만 interface 입력과 미지원 안내는 viewport 아래에 있어 이 PNG로 공개 완료를 증명할 수 없다. UIA geometry가 in_owned_content라고 기록하더라도 실제 body clip의 증거로 대신하지 않는다. disabled 필드 강제 focus는 하지 않으며, 확장 후 critical 안내 공개/사용자 스크롤 경로를 실제 raster로 확인하는 후속 작업이 필요하다. 현재 UI/UX 인수는 미완료다.

### 확장 공개 후보 주 담당 검토

`/tmp/dlpad-network-one-shot-reveal-v2/accessible.rs.patch`를 읽었다. open 전환을 추적하고 애니메이션 완전 열림 이후 한 frame의 모든 재pass에서 같은 scroll 요청을 유지하여 discarded pass에서 의도를 잃지 않는 후보다. 기존 긴 안내 의미는 보존한다. 해당 전체 diff에는 최신 inspector400을300으로 되돌리는 부분과 새 ready 회귀를 삭제하는 관련 없는 hunk가 포함돼 있어 전체 적용하지 않는다. 통합 시 NetworkReveal 타입·Candidate field/init·NETWORK_OBSERVATION 호출의 독립 hunk만 추출하고 최신 UI/test를 보존한다. 현재 빌드97955 실행 중이므로 source 수정은 보류한다.

미지원 안내를 비활성 입력보다 먼저 배치하고, 확장 전환 후 애니메이션이 끝나면 안내를 한 번 공개하는 후보를 통합했다. 같은 frame의 재pass는 요청을 유지하며 다음 frame에서 소비한다. 사용자 wheel로 재arm하지 않는다. 최신400px inspector와ready회귀를 보존해 독립hunk만 병합했다. 실제 painted text/후속wheel/Candidate통합 회귀를 추가했으나 아직 컴파일·실행되지 않았다.

network-reveal-geometry whole는 새 metrics 테스트가 stop_wake_worker_tests에 배치되어 network_text_shapes/stop helper를 찾지 못해 compile101로 실패했다. 원 로그를 보존하고 기존 metrics tests 모듈로 옮겼다. 아직 재검증되지 않았다.

strict Clippy가 새test3곳의collapsible_if로실패해let-chain으로수정했다. wheel 회귀offset은 재pass당기록에서frame의최종pass한번기록으로교정해 frame45 wheel과frame65 idle을정확히비교한다. 검사조건/lint는완화하지않았다.

### 실제 Context 회귀 통과

windows-msvc-network-reveal-finalpass-whole-20261012/test.log에서 actual_network_open_reveals_reason_once_and_later_wheel_stays_put, actual_candidate_settings_open_paints_complete_unsupported_reason, actual_unsupported_reason_precedes_inert_input_at_200_percent가 두 app harness 모두 ok로 실행됨을 확인했다. 안내 TextShape의 실제 visual bounds와 최종 clipped shape, discarded pass 후 공개, frame45 wheel 후frame65 offset유지를 검사한다. 이는 실제 OS pointer·Windows native raster·physical IME/reader 증거가 아니다. 전체 release 빌드는 아직 진행 중이다.
