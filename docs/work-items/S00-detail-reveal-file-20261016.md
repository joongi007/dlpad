# S00 로컬 영상의 탐색기 파일 선택

- 상태: done-scoped / 사용자 검수 대기
- 분류: feature
- 우선순위: P0
- 요구사항: R03, R04, R08, R11
- 담당/수정 범위: field_reveal — crates/app/src/library_open.rs, external_open_broker.rs, media_detail_open.rs 및 이 작업 기록만
- 선행 작업: 기존 retained media preview 및 외부 열기 broker. Root가 catalog의 MEDIA_REVEAL_FILE/MEDIA_REVEAL_HELP 추가와 전체 통합을 담당한다.

## 사용자 결과

다운로드 화면의 로컬 영상 상세에서 탐색기를 열고 원 파일을 선택한다. 파일 이동·복사·전송은 하지 않는다.

## 계약과 범위

AGENTS → README → PRODUCT → CONTEXT → 관련 요구사항, ADR 0009/0010, DESIGN, ARCHITECTURE를 따른다. Kind::RevealFile 추가는 Root와 합의한다. 기존 원 파일/부모 pin 및 fresh identity 검증, 취소·기한·Job·Default desktop·동일 사용자/세션·외부 Explorer identity gate를 유지한다. shell command 생성과 앱 Job의 Explorer 직접 spawn은 금지한다. 중앙 catalog/engine/Cargo/lock 및 다른 UI 파일은 수정하지 않는다.

## 인수 조건

- 로컬·사용 가능·idle에서만 기존 secondary 버튼과 같은 규칙으로 한 번 요청한다. 원격·busy·blocked에서는 요청하지 않는다.
- retained 원본 파일을 선택하며 오래된 identity/변경·취소·시간 초과·broker 불일치에서 외부 동작을 하지 않는다.
- mock handoff로 문자 그대로의 한글·공백·기호 경로와 RevealFile 경로를 검증한다. 실제 Context에서 Light/Dark, 100/200% 동작·줄바꿈·원 focus/reveal 계약을 검증한다.
- 자동 시험에서 실제 Explorer나 사용자 desktop을 열지 않는다. Windows native API 실제 동작 및 시각 사용자 인수는 별도 미검증으로 기록한다.

## 검증과 handoff

구현 후보 작성 및 rustfmt 완료. Root 승인 후 Kind::RevealFile을 추가했고 unowned accessible_library.rs는 비교만 사용하므로 exhaustive match 변경은 없다. RevealFile은 Job 여부와 무관하게 broker를 요구한다. Request는 원 파일과 실제 부모의 일치를 검사한다. 원 파일 PIDL을 생성하고 SHOpenFolderAndSelectItems(cidl=0, flags=0)를 한 번 호출하며 PIDL은 RAII로 해제한다. 기존 Explorer capture/identity 검사가 실패하면 API에 도달하지 않는다.

이 API는 Shell service에 선택을 위임하는 공개 API이며 Bound.dispatch의 원격 COM 메서드를 호출하는 것은 아니다. 기존 외부 Explorer 검증과 명시적 선택 요청을 유지하는 범위로 Root가 승인했다. API 내부 OS dispatch의 프로세스 생성/실제 선택 완료는 자동 시험으로 입증하지 않는다. 성공 표시는 handoff이며 탐색기 화면/선택 ack가 아니다. 동기 native 호출은 기존 외부 열기와 마찬가지로 hard wall 취소가 아니고 반환까지 StopAck가 대기한다.

Microsoft API 계약: [SHOpenFolderAndSelectItems](https://learn.microsoft.com/en-us/windows/win32/api/shlobj_core/nf-shlobj_core-shopenfolderandselectitems). 새 의존성 없이 Shell32의 opaque PIDL 포인터 FFI만 사용한다. cfg(test)/--fixture/private desktop에서는 원 자동 차단을 유지한다.

준비된 회귀: mock literal-path RevealFile/잘못된 부모/다섯 Explorer identity 거절, retained 원 파일 선택 및 변경/취소/기한/자동 차단, 실제 Context 세 버튼 일회 동작·비활성·원격·Light/Dark·100/200%·전체 glyph 노출·focus/no-resnap. 아직 실행하지 않았다.

이 담당은 Cargo/native/GUI를 실행하지 않는다. Root가 통합 뒤 실제 컴파일·회귀·정적 검사를 실행한다. 새로운 의존성·schema·프로세스를 추가하지 않는다.


### Root 최종 통합 — 2026-10-17

최종 검수본은v35다. Windows whole35 fmt·strict workspace/all-targets clippy·test·release 실제0,1679/0/42·43harness·source675·3PE·lock을 Root가 대조했다. 실제 CPU renderer104프레임/unpainted0·Light/Dark100/200% PNG4개를 직접 검토했다. 새 패키지 전체CRC/SHA와 private 설치·제거·동일위치 재설치·cold 확인(수동 재검증0회)·설치 앱 자연0/Job0을 확인했다. [통합 검수](../validation/ui-review-v35-20261017.md), [최신 실행](../development/windows-user-preview.md).

Explorer 실제 선택과 최종 사용자 UI 승인은 남는다. 자동 시험에서 사용자 desktop·clipboard·외부 viewer를 열지 않았다. 버전 비교·기존 앱 자동 교체는 후속이며 최신 후보 확인에는 새 폴더 또는 portable을 사용한다. 기존 .git은 보존하고 Git조회exit128을 유지한다.


### 사용자 실사용 오류 후속 — 2026-10-17

v35의 ‘사용자 검수 대기’ 상태에서 기본 앱 성공/탐색기 선택 실패가 제보됐다. 원 native API의 실제 입력 경계를 시험하지 않은 공백을 확인했다. canonical verbatim 경로를 Shell 파서에 그대로 넘기면 거절되는 원인을 별도 합성 native 실험으로 재현했고, [후속 수정 작업](S00-reveal-native-failure-20261017.md)에서 원 권한을 유지한 ordinary alias와 실제 parser 회귀를 구현했다. 위 v35의 ILCreate 구현과 검수 결과는 고정 snapshot 기록이며 최신 수정 완료를 뜻하지 않는다.
