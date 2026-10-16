# S03: Windows 실제 장시간 soak 준비와 실행

- 상태: done-scoped — frozen Windows 실제8시간 합성 전송 완료
- 분류: verification
- 우선순위: P0
- 요구사항: R04, R08, R11, R13
- 담당/수정 범위: root crates/engine/tests/windows_download_soak.rs·실행·통합·scripts·docs; release_preparation은 /tmp 조사·후보만 준비한다. Cargo/schema/lock과 제품 interface 변경은 root 담당.
- 선행 작업: 실제 HTTP403/Range 복구·writer112·8시간길이 seek/cut·현재 Windows whole proof

## 사용자 결과

길이가8시간인 합성 영상의 짧은 연산과 실제8시간 wall-clock 작업을 구분하고, 장시간 실행의 메모리·프로세스·DB·중단 및 최종 파일 정합성을 검증한다.

## 계약과 범위

별도 가상 화면과 앱 소유 임시 폴더·합성 source만 사용한다. 필요한 테스트 서버는127.0.0.1만 사용한다. 원본 release/proof를 create-only 복사해 불변 SHA에 연결하며 장시간 관측 동안 현재 worktree가 발전하는 것을 금지하지 않는다. 새로운 source로 소급해 인수하지 않는다. 원 프로세스/creation/HANDLE/Job과 bounded 로그·진행률·표본을 사용한다. 물리 sleep·전원 차단·실제 사이트 timeout은 별도 범위다.

## 인수 조건

실제 monotonic 경과시간·표본/상한·정상/취소/최종 DB와 파일을 관측하고 원 Job 종료·합성 sentinel 보존을 확인한다. 압축 시간이나 계획만으로8시간을 완료로 표시하지 않는다. 장시간 observer의 live handle과 terminal state를 유지해 세션 재개에서도 중복 재시작하지 않는다.

## 검증과 handoff

현재 source/CLI/timeouts와 실제 가능한 observation interface를 먼저 확인한다. OS shutdown 뒤 관측이 사라지면 완료나 정상 종료로 추정하지 않는다.

Windows 실제 전송을 pacing하는 ignored 시험2개를 통합했다. preflight는72초/formalfalse, 정식 시험은28800~28860초/formaltrue이며 성공 뒤 기다리는 방식이 아니다. 실제 body의 마지막 byte를 monotonic deadline 뒤 전송한다. HEAD/짧은Range·metadata 초기단절·Windows accepted blocking socket, bounded headers/buffers/requests와 stop/thread join을 유지한다. 실제 DB/ProgressFeed·원 artifact SHA·DB 재열기·도구/합성 sentinel·최대480개/512KiB 표본을 검증한다. 표본 메모리는 시험 runner의 실제 working set이며 앱/외부 child 전체 비용으로 확대하지 않는다.

현재 whole1015/0/28에서 새 시험2개는 ignored로 컴파일됐으며 아직 장시간 인수 PASS가 아니다. 원 host의7200초 제한을 변경하지 않고 별도32400초 test-only host 후보를 작성했다. 실제 소유ACL·원 input copy·ffprobe길이·guard를 짧은 실행으로 확인한 뒤 별도 긴 명령을 시작한다. 첫 collector는 Cargo mixed-separator 경로를 거절했고, 두 번째는 merged guardian JSON/stderr 파싱으로 실패했다. 모두 실제 compiler가 끝난 뒤의 수집 오류이며 원 실패 로그·receipt·후보를 보존했다. guardian은 exact Cargo argv와 검증된 target directory의 고정 출력에 묶는 세 번째 collector로 preflight 실행 중이다. 새 의존성·Cargo/schema/lock 변경은 없다.

## 2026-10-11 실제 장시간 시험 시작

세 번째 사전 시험은 실제 72.0002307초 전송과 DB 재개방을 통과했다. 루트가 8개 작업의 원시 Job 종료 기록, 원본/복사본 SHA·크기, 현재 소스 일치를 재검증했다. `windows-msvc-wall-soak-preflight-third-20261011/root-review.json`은 해당 범위 PASS_SCOPED이며 8시간 완료를 뜻하지 않는다.

`windows-msvc-eight-hour-wall-soak-20261011`에서 28,800초 실제 전송을 시작했다. 소유 실행 세션은 55316이며 합성 fixture와 loopback만 사용한다. 고정 복사본의 도구·테스트를 실행하므로 이후 앱 UI 변경과 독립적이다. private root ACL/파일 ID 재확인과 자연 종료 Job0 기록은 확보했다. 최종 전송·원본 보존·DB·주기 샘플 검토 전에는 장시간 시험을 완료로 표시하지 않는다. WSL/호스트 중단 시 미완료로 보존하며 자동으로 성공 처리하거나 중복 실행하지 않는다.

### 실제8시간 종료 및 독립검토

기존55316 collector는LONG_RUN_REVIEW_REQUIRED로자연종료했다. 주 담당이준비된readonlyreviewer와pure5를검토/실행했고root-review.json에결과를보존했다. 실제전송 28800.0003048초, elapsed 28805.9653367초, samples 474개/105976bytes, 고정원본 20개보존; 원본Job natural0/actualtestargv/표본간격상한/DBintegrity재개방/Downloaded같은job·attempt·artifact/완성파일SHA검토PASS. 고정된이전runtime합성전송범위이며현재UI/실사이트/물리절전복귀/공개릴리스인수로확대하지않는다.

2026-10-16 마감감사에서 원종료/rawWindows시험·20불변입력·readonlyDB/완성파일/표본474를 재대조했다. 상단 상태를 실제완료에맞춰정정했고 이전중간관측/실패는보존했다. UI변경마다 새8시간을 반복하지 않는다. [재대조 기록](../validation/evidence/current33-p0-closeout-root-20261016/WINDOWS-8H.md).
