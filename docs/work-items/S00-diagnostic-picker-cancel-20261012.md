# S00: 진단 저장 위치 선택 취소

- 상태: completed
- 분류: UX
- 우선순위: P0
- 요구사항: R08, R13
- 담당/수정 범위: root — /tmp/dlpad-diagnostic-picker-cancel-native-20261012 시험 후보, 이 문서와 validation evidence
- 선행 작업: 최신1193 whole 및 저장 UI 통과

## 사용자 결과

저장 위치 선택창에서 취소하면 기존 경로 초안을 그대로 유지하고 다시 조작할 수 있다.

## 계약과 범위

현재 원 앱 PID/creation과 private desktop의 dialog→proxy→main 소유 관계를 대조한다. 정확 Cancel2 control의 PID/TID/native child/control ID를 확인한 뒤 그 control만 클릭한다. 전역 입력과 사용자 화면 fallback은 없다. 제품 코드와 공유 schema/Cargo는 변경하지 않는다.

## 인수 조건

취소 뒤 modal이 닫히고 입력·선택 버튼이 재활성화되며 경로가 정확히 유지된다. 이후 새 선택과 명시 저장·기존 파일 거절도 정상 작동하고 앱은 natural exit/Job0로 종료한다. Light/Dark 실제 화면과 별도 원 증거 대조로 검증한다.

## 검증과 handoff

시험 후보를 준비했으며 실제 실행·독립 검토는 아직 미완료다. 작은 창/200%와 owner-close는 별도 검증이다.

- 첫 Light5272/Dark73542 exit2: compile 원 로그에 WSL UtilBindVsockAnyPort socket failed1. GUI 시작 전 호스트 호출 실패이며 제품 취소 실패로 해석하지 않는다. 실패 evidence를 보존하고 새 v2 output에서 허용된 native 실행 권한으로 재시도한다. 문서검사19941 exit0:957 documents.

- v2 Light39512/Dark57737 exit0; 독립 검토74191/45734 exit0.
 actual Cancel2의 modal 종료·빈 초기 경로 유지·입력/선택 재활성화를 확인한 뒤 새 선택/명시 저장/기존 파일 보존/natural Job0까지 통과했다. 원 evidence의 root-independent-review.json으로 대조를 보관했다. root는 Dark picker-cancel-return.png에서 빈 경로·선택 활성·저장 비활성의 정상 상태를 직접 확인했다. 비어 있지 않은 편집 초안의 취소 보존은 아직 실제 GUI로 검증하지 않아 작업 완료로 판정하지 않는다.

- 후속 한글/emoji 비어있지않은 경로 초안을 실제 입력하고 fresh provider readback을 확인한 뒤 Cancel2로 취소하는 후보를 /tmp/dlpad-diagnostic-picker-cancel-draft-native-20261012에 준비했다. 원초안 exact보존·파일생성없음 및 후속원저장시나리오를 유지하며 Light43148/Dark83225 실행 중이다. 완료로판정하지않는다.

- 비어있지않은 초안 Light43148/Dark83225는 exit2로 실패했다. Cancel 조작 전 owned-WM_CHAR 입력 readback에서 expected UTF16 length121/actual119 불일치가 발생했다. 두 실행의 gui.log 원 bytes를 보존했다. Emoji surrogate 입력 누락 가능성을 실제 관측한 상태이며 원인을 확정하지 않는다. 강제 owned Job cleanup으로 종료했으므로 natural-exit 인수도 아니다. 다음 작업은 정확 입력 경로를 최소 재현하고 제품/시험 전송 중 어느 쪽 문제인지 구분하는 것이다. 성공한 빈초안취소/파일선택emoji 인수로 이 실패를 덮지 않는다.

- 진단 가설: (1)시험F24 envelope가 UTF16 surrogate 사이를 끊음, (2)winit 변환, (3)widget 필터. vendor/winit keyboard.rs168~224는 key event의utf16parts를연속WM_CHAR에서합치며 다음키메시지가CHAR가아니면finalize한다. 기존시험은 surrogate마다F24 down/CHAR/up을전달했다. 제품source는수정하지않고두UTF16 unit을같은envelope로전달하는/tmp v2 후보를실행해가설1을검증한다. 이는물리IME인수아니다.

- v2 Light44703/Dark96133 exit0: 제품source/CPU는그대로이고UTF16 surrogate쌍만같은F24 envelope에전달했다. expected/actual UTF16 length121과SHA가두테마에서정확히일치하고비어있지않은한글/emoji 초안도취소후동일했다. 후속새선택·명시저장·기존파일거절·자연종료/Job0도통과했다. 최초실패는시험전달구성으로확인했으며실제물리IME지원완료로확대하지않는다. root는두picker-cancel-return.png를직접확인했다. 독립58722/11583은진행중이다. 문서검사5418 exit0:962 documents.

- 독립58722/11583 exit0: 한글/emoji 초안exact보존·원초안파일부재·소스/pins/raw/readonly DB/natural Job0를대조해두evidence/root-independent-review.json으로통합했다. 이수직작업은960640/app100%의저장창취소범위로완료한다. owner-close·MPD취소·물리IME·fullpath hover·전체UI/UX는별도잔여범위다.
