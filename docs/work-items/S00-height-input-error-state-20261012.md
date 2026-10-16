# S00: 품질 높이 입력 오류 상태

- 상태: in-progress
- 분류: UX
- 우선순위: P0
- 요구사항: R05, R07, R08
- 담당/수정 범위: diagnostic_cache — /tmp 후보; root — crates/app/src/accessible_options.rs 통합, 이 문서
- 선행 작업: source-first UI 전체·native 검사 종료 후 source 변경

## 사용자 결과

잘못된 품질 높이를 입력하면 오류를 해당 입력과 함께 읽고 식별하며, 값을 고치면 오류 상태가 사라진다.

## 계약과 범위

DESIGN.md와 GTMUX Input의 오류 의미 연결을 따른다. Height 필드만 연결하며 core validation·논리 ID·caret·disabled 정책은 유지한다. 범위 및 네트워크 입력은 후속이다. 에이전트는 프로젝트 파일을 수정하지 않는다.

## 인수 조건

실제 DownloadForm.show에서 invalid→valid의 입력 node invalid/description과 scoped 오류 테두리를 검증한다. 정상·focus·disabled 상태를 보존한다. native provider 및 raster는 별도 확인한다.

## 검증과 handoff

현재 높이 오류는 별도 label이며 입력 의미와 연결되지 않는 코드 불일치를 확인했다. 후보 준비 중이고 실제 제품 변경·native 인수 완료는 아니다.

- root는 실제 Form.show 테스트만 먼저 통합했다. RED86970 exit101: 실제 height0 입력 후 node invalid=None/기대True로 실패. compiler/collector 오류가 아니며 원 source 불변·Job 자연 종료 확인.
- 원 quality parser 결과를 해당 height node의 Invalid/description과 정확한 입력 rect의 danger stroke에 연결했다. 정상 값으로 고치면 의미·테두리를 제거하고 같은 ID/선택·caret/disabled 정책을 유지한다. helper의 기존 return() 호출 계약은 wrapper로 보존하며 새 field_response를 height에서만 사용한다. core/schema/lock/dependency 변경 없음.
- 옵션 회귀 GREEN 실행 중. 새 전체 검사와 native provider/raster 인수는 아직 남아 있다.
- GREEN66636 exit0: 두 UI harness 각각 옵션10회귀 통과. 실제 AccessKitFocus→CtrlA/height0→InvalidTrue/정확한 description·오류 stroke→1080→오류 제거·동일 focusID/caret4→disabled 입력 차단 포함. 원 parser를 사용하며 별도 parser 복제 시험이 아니다. source 불변·Job 자연 종료 확인. 현재 input-focus-error 전체 검사를 시작했고 native provider/raster는 후속이다.

- input-focus-error-whole-v2 전체28685 및 독립84941 exit0: fmt/clippy/test/release,43 harness/1161성공/0실패/30ignored와 원로그/source/artifact 검증 통과. CPU9,709,568바이트 SHA `b489e5932b227ee91f39eeeed345a5a493dffcf16a60c42f8882cacb5b58cb53`; source-first 대비1,024바이트 증가. 의존성/schema/lock 변경 없음.

- native Light75011/Dark7682 시작. locked accesskit_windows0.29.2의 properties 목록에 IsDataValidForForm mapping이 없어 실제30103 속성은 미지원일 가능성을 확인했다. native 결과를 기다리며 AccessKit 내부 Invalid 통과를 Windows 속성 성공으로 대체하지 않는다.
- v1 Light75011/Dark7682는 observer의 오류 문구 기대가 실제 core ValidationError와 달라 실패했다. 실제 문구는 `품질 높이는 144~4320 범위입니다`이며 기존 실패·원본을 보존했다. 교정 v2 Light57255/Dark23693은0/1080의30103을 모두false로 관측해 OS bridge 실패를 확인했다. 30159는 오류→빈 문자열로 복구됐다. 두 앱은 실패 정리에서 종료돼 자연 종료 성공으로 계산하지 않는다. 별도 Windows provider work-item에서 해결한다.
- root가 error_fg_color=foreground인 테마 계약을 확인했다. 오류 테두리는 별도 DANGER_MARKER로 바꿔 본문 가독성과 오류 마커를 구별한다. metadata/parser/입력ID는 그대로이며 marker 수정 후 옵션 회귀를 실행했다. 새 전체·native 인수는 후속이다.
