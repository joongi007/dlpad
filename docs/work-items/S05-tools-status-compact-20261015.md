# S05: 작은 도구 화면의 상태와 설치 동작

- 상태: in-progress
- 분류: bug
- 우선순위: P0
- 요구사항: R05, R08
- 담당/수정 범위: installer_batch accessible_tools.rs 단일 scratch 후보·본문서/B.tools-status-compact-v1; Root catalog/통합
- 선행 작업: 현재3slot cache·Root Tools12PNG 직접검토, common ui_components::help

## 사용자 결과

800폭/200% 화면에서도 상태·현재선택·설치/갱신을 위에서 찾는다. 긴 역할/출처·cacheauthority/최근목록설명은 상태옆 공통도움말로필요할때 읽는다. 필수상태/오류/라이선스·출처 세부/용량/행동gate는보존한다.

## 계약과 범위

새 catalog 두문구는Root소유: TOOL_DETAILS_HELP_TITLE/TOOL_CACHE_STATUS_SHORT. stable tool-key help/nativeprovider·32행을재사용한다. query/cacheauthority/backend/actionworker/generation 변경없음. Live/S/Cargo/Rootmetrics/공유UI수정금지; Root sourcefreeze 때문에scratch만작성한다.

## 인수 조건

actualContext800200/lightdark에서footer fence위로status/mainaction fullbounds+paintclip을확인한다. 긴 optionaltext는공통help description에보존한다. 기존cachedtab/원rowID/provider/세대·freshaction 회귀를보존하며표시 위치만교정한다. 물리GUI/IME/전체P0 완료를Context로대체하지않는다.

## 검증과 handoff

원본backup/pin후후보작성을시작했다. 현재Live변경0/native0/Cargo0. Root ACK후통합한다.

## Root 승인 반영과 검증 대기

Root ACK 후 원 Live 핀과 후보 핀의 일치를 확인하고 accessible_tools.rs 한 파일만 반영했다. file-only rustfmt 1.94 exit0이며 후보와 최종 Live bytes가 같다. 최종 183133바이트 SHA `a785c5f77a8f044febdc1e8b4c89c76b76e2b6d40ef50e47790b0eb11a3ce1cc`다. 이후 source freeze를 유지한다. Windows Context/GUI 시험은 아직 실행하지 않았으며 상태는 in-progress다. B/tools-status-compact-v1/adoption.json에 대조 결과를 기록했다. 공유 source/S/Cargo/원 evidence는 수정하지 않았다.

Root 선택 GREEN 실제 검수: managed-tool-isolated-native-value-green-v1-20261015에서 입력 9, 엔진 전송 4, 진행률 Context 1(64조합), 도구 compact Context 1(6장면)이 모두 통과했다. 각 명령 exit0/소유 Job0·private desktop 종료·input desktop 불변 및 Live/후보 source 불변을 Root가 원로그와 receipt로 대조했다. 전체 Windows 네 명령은 새 whole-v3에서 실행 중이며 실제 GUI/새 설치본은 아직 미완료다.

whole-v4의 fmt는 통과했으나 새 시험의 중첩 조건문 한 곳이 strict clippy(collapsible_if)에 거절됐다. 원 실패를 보존하고 해당 조건만 let-chain으로 정리했다. 제품 정책·인수 조건을 바꾸거나 lint를 허용하지 않았다. 담당 파일 183,117바이트, SHA256 616b30ff9b27e05913460e23a35c4d4f624da60a7183027ec11927d795bfff17로 동결했다. 새 whole-v5에서 fmt와 strict clippy가 통과하고 test는 1,591/0/42 ignored/43 harness로 끝났으며 release는 아직 실행 중이다.
