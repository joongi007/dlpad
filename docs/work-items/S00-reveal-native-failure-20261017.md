# S00: 기본 앱 열기와 달리 탐색기 선택이 실패하는 오류

- 상태: done-scoped / 사용자 선택 화면 검수 대기
- 분류: bug
- 우선순위: P0
- 요구사항: R03, R08, R11
- 담당/수정 범위: Root — external_open_broker.rs, library_open.rs, 필요한 중앙 catalog/상세 오류 표시, native 검증과 최신 검수 문서. p0_audit는 읽기전용 COM 감사.
- 선행 작업: v35 사용자 실사용에서 기본 앱 성공/탐색기 선택 오류 제보

## 사용자 결과

같은 로컬 파일에 대해 탐색기에서 파일 선택 요청이 정상 전달되고 실패 원인은 구분한다. 기본 앱 열기와 파일 보존/취소/수명 계약을 유지한다.

## 계약과 범위

AGENTS→README→PRODUCT→CONTEXT→R03/R08/R11→관련 ADR0009/0010 및 ARCHITECTURE 계약을 따른다. 기존 .git 조회exit128을 기록하고 보존한다. 첫 native loop는 임시 합성 파일의 PIDL 생성만 private desktop에서 차등 실행한다. 실제 사용자 탐색기/visible desktop은 열지 않는다. COM 감사와 PIDL parse를 분리해 경계별로 진단하며 실패 재현 전에 임의 권한 완화나 우회 fallback을 추가하지 않는다.

## 인수 조건

- 같은 canonical 로컬 파일에서 file-open/reveal 분기 차이를 재현하고 실제 API 반환값을 기록한다.
- 원 파일/부모 pin, cancel/deadline, Default desktop와 외부 앱 수명 계약을 보존한다.
- 실패 원인을 구분하는 회귀와 Windows 실제 검사·새 검수 패키지·문서 갱신을 수행한다.
- 실제 사용자 Explorer 선택과 private API parse/mock의 검증 범위를 구분한다.

## 검증과 handoff

진행 중. 사용자 오류의 정확한 문구/코드는 아직 수집되지 않았다. 이를 임의로 확정하지 않고 준비/캡처/COM/PIDL/선택 요청을 분리해 좁힌다. native probe는 새 프로젝트 소유 경로만 사용하며 전체 P0나 OS 검수를 완료로 표시하지 않는다.

### 원인과 실제 피드백 루프

Windows private native 합성 실험 v2/v3에서 ordinary 경로119/371UTF16은 ILCreate/SHParse 모두 성공했고, 같은 원 파일의 verbatim canonical 경로123/375UTF16은 ILCreate NULL·SHParse E_INVALIDARG(0x80070057)로 두 차례 실패했다. 창 열기/선택 요청0이며 원 Job 자연0을 확인했다. 첫 v1은 host argv 누락으로 실행되지 않은 실패로 보존했다.

실제 production broker 인수 경계에 회귀를 먼저 추가한 RED는 canonical spelling이 그대로 전달되어 실패했다. 수정 후 GREEN은 관련16시험이 통과했다. 새로운 실제 native Pidl parser는 한글·공백·기호와260자 초과 파일의 파싱→PIDL→파일경로 roundtrip을 검증한다. 다른 파일/부모로 해석되는 trailing-dot/trailing-space alias는 원 pin/nativeID/canonical entry 대조로 거절한다. 원본 파일 권한은 canonical spelling과 retained HANDLE에 남고 Shell spelling은 검증한 동등 표기만 사용한다. File/ChooseApplication 분기와 Default desktop·Job·외부 Explorer gate는 바꾸지 않는다.

p0_audit의 추가 read-only COM probe는 원 private desktop/기존 같은세션 Explorer 조건에서 VT_EMPTY와 VT_I4(0) 모두 S_OK와 dispatch를 반환했다. 따라서 현재 관측에서는 위치 variant 차이를 원인으로 채택하지 않는다. private desktop의 GetShellWindow는0이며 cross-desktop HWND의PID관측불가를 제품capture실패로 표현하지 않는다. 사용자 창 조작·Execute/Navigate/Select/desktop 전환0이다. 원본 report/COM binding 입력은 별도 evidence에 보존한다.

field_reveal은 ui36-integration-prep 아래 whole/review/package helper만 담당했고, 원35 helper/base160 pin을 보존했다. 제품 두 파일과 schema/Cargo/lock 및 통합 실행은 Root만 담당한다. 최초 root2파일 변경으로 source675를 동결했다. 전체 Windows 검증·최신 패키지는 진행 중이다.


### 최종 Root 통합

Windows whole36 실제 fmt·strict workspace/all-targets clippy·test·release0,1685PASS/0FAIL/42ignored/43harness·source675·3PE·lock과 현재 graph258/notice456·설치 ZIP7/portable467 전체 CRC/SHA를 대조했다. native 차등 실패2회와 RED→GREEN16, actual production parser의 short/long Unicode roundtrip이 근거다. [통합 검수](../validation/ui-review-v36-20261017.md), [최신 실행](../development/windows-user-preview.md).

원인: canonical verbatim 경로를 Shell namespace parser에 그대로 보낸 것이다. 방지책은 fake provider만 통과시키지 않고 실제 production handoff 인수와 native PIDL parse를 합성 파일로 회귀하는 것이다. Default desktop/외부 Explorer identity gate는 그대로이고 실제 선택 화면의 성공은 사용자 검수에 남는다. 최초 exact 사용자 오류 문구는 아직 수집되지 않았으며 모든 환경의 실패 원인을 동일하게 확정하지 않는다.
