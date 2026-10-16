# S03: moved leader terminal 오류 반복 진단

- 상태: completed (반복 관측 완료, 원래 실패 미해결)
- 분류: regression-diagnosis
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: stress_recovery; 신규 scripts/guardian_moved_leader_regression.py 및 이 문서
- 선행 작업: cookie guardian begin gate 전체 engine 시험의 기존 moved leader TerminalUnconfirmed 실패

## 사용자 결과

300ms 계약을 바꾸지 않고 실제 같은 test/helper 사본20회로 기존 ungated 회귀를 재현한다.

## 계약과 범위

source/실행파일 SHA를 기록하고 test/helper를 owned /tmp adjacent0500 사본으로 고정한다. 정확한 기존 시험명만 argv로 실행한다. 회차별 log128KiB/시간10초 및 known descendant inventory64개 상한이다. descendants는 이번 driver의 /proc children에서 PID/start/parent를 확인하며 외부 PID를 검색하거나 종료하지 않는다. 시간초과시 정확한 자체 driver pidfd만 종료하며 불확실한 자손은 보존해 root에 보고한다. 제품/Rust 시험/timeout/typed오류/기존 실패 로그/8h는 수정하지 않는다.

## 인수 조건

총20회 원시 exit/output/hash/시간 보존, 시험 실제1개 실행 확인, 관측된 owned identity terminal 비교 및 unknown survivor 명시. 실패를 성공으로 바꾸거나 timeout을 늘리지 않는다.

## 검증과 handoff

작성 중. 원시 실패는 cookie-guardian-begin-gate/engine-first.log에 보존된다. source상 leader는 부모 group으로 이동하지만 원 group ESRCH는 정상으로 처리되고 unreaped positive PID에 SIGKILL한 뒤 wait/reap한다. 이어 control_reader.finish 실패도 GU/TerminalUnconfirmed가 되므로 오류만으로 생존 child/그룹 정리 실패라고 확정할 수 없다. 실제 반복 증거가 필요하다.

## 실제 반복 결과

`python3 scripts/guardian_moved_leader_regression.py target/debug/deps/guardian-a79b357dd6ca170f target/debug/dlpad-process-guard /mnt/d/project/dlpad/docs/validation/evidence/guardian-moved-leader-first` 실제 exit0, 20/20 기존 시험 통과. 첫 collector는 main task children만 읽어 worker thread 자손을 놓쳤다. inventory0은 정리 독립 증거가 아니며 원시를 보존했다.

동일 명령의 새 output `guardian-moved-leader-second`에서 모든 소유 task의 children을 최대64thread로 읽도록 보강한 뒤 실제 exit0,20/20 통과, TerminalUnconfirmed0, 회차0.532~0.548초였다. helper+tool 각2개씩 총40 exact PID/start identity가 관측됐고 종료 뒤 모두 부재, unknown survivor0이다. 본 프로브는 process signal을 실행하지 않았고 자체driver 시간초과도 없었다. copy temp 정리와 source/binary 전후 SHA 불변을 확인했다. 원시 engine-first.log의 전체 병렬 시험 실패는 여전히 미재현이며 실패 해결 완료가 아니다.

고정 driver SHA `7b6130b92e6daea4c6deab681d2e179c2f246e06d3d94aa299a291a53a84e297`, helper SHA `49f9f1359f702efdc6df276247bf7b86c603cb43d618e96e4a33f7a08e9c0c86`이다. helper/process source는 root cookie-guardian-begin-gate/native-review.json과 일치한다. 신규 script SHA `479f0a82ac87da3e2710603b2d26ae1b47b2d502492361a8dbf0d16ac80f8bd6`로 freeze했다. Cargo나 기존 Rust 파일 수정은 없다.

소스 기반 미확정 가설: moved leader는 원 group TERM이 ESRCH라 실제 positive PID kill이 guardian의200ms TERM grace 뒤 발생한다. 앱의 동일200ms grace 만료에서 마지막 try_wait 이후 ownership.signal(true)의 socket write가 guardian reader 종료/GC와 경쟁하면, GC 확인 전에 반환한 I/O 오류가 TerminalUnconfirmed로 분류될 수 있다. 반복 stdout는 실제 control wire와 원 I/O kind를 보존하지 못하므로 이 가설을 원인으로 확정하지 않는다. 다음 root 단계는 원래 병렬 실행 조건에서 raw control/정확한 실패 단계 관측으로 재현하는 일이다. timeout300ms나 오류 인수 기준을 바꾸지 않는다.

## 후속 전체 suite 준비

wrapper의 `--suite`는 기존 guardian9개 전체를 libtest 기본 병렬 조건으로 --nocapture 실행한다. 반복은20회이며 300ms 시험 계약은 그대로다. 새 debug-only 실패 stage/kind/raw_errno trace driver를 root가 제공할 때까지 실행하지 않는다. 실행 예시는 `python3 scripts/guardian_moved_leader_regression.py NEW_DRIVER NEW_HELPER /mnt/d/project/dlpad/docs/validation/evidence/guardian-moved-leader-suite-NEW --suite`다.

known PID/start64개 또는 task64개 관측 상한 초과는 inventory_error와 inventory_verified=false로 기록하고, native test 결과와 별도로 미검증 처리한다. 상한 때문에 외부 PID 검색/정리 범위를 넓히지 않는다. source/copy/original binaries SHA를 전후 확인한다. suite 일부 기존 fixture는 env!(CARGO_BIN_EXE_dlpad-process-guard)에 내장된 원본 helper 경로를 직접 사용한다. 따라서 adjacent 고정 copy만으로 suite의 모든 helper 경로를 고정했다고 주장하지 않는다. root는 실행 중 원본 build helper도 동결하고, wrapper의 원본 SHA 전후 검사와 함께 확인해야 한다. 이번 후속은 구문 준비뿐이며 기존 driver로 추가 실행하지 않았다.

Root는process.rs에명시 DLPAD_TEST_CLEANUP_STAGE_TRACE=1/debugbuild에서만 fixed stage/kind/raw_errno를출력하는진단을추가한다. argv/tooloutput/path/PID/credential은출력하지않고오류판정을변경하지않는다. suite재현후일시진단코드제거또는별도확정검토예정.

## 새 trace driver 전체 병렬 suite 실제 결과

`python3 scripts/guardian_moved_leader_regression.py target/debug/deps/guardian-a79b357dd6ca170f target/debug/dlpad-process-guard /mnt/d/project/dlpad/docs/validation/evidence/guardian-moved-leader-suite-trace-first --suite --cleanup-stage-trace` 실제 exit0, 기존9tests ×20회 모두 통과했다. trace 환경 변수는 subprocess env copy에만1로 설정했으며 부모/전역 환경은 변경하지 않았다. 실패가 없으므로 stage/kind/raw_errno 실패 출력은 없었다.

회차1.055~1.108초, 관측 소유 PID/start 총679개(회차33~35개)가 모두 종료 뒤 부재였다. inventory_verified는 모든 회차 true, 상한 초과 및 unknown survivor0이다. source/copy/original driver/helper SHA 전후 불변, owned copy temp 제거를 확인했다. root 지정 새 고정 driver SHA `a59abd8256264e939e8374bd4ef2b99d3fb801e27e59ca9aa9badac3cdda9537`, helper SHA `9a09315acf47a52e88db41d5828f64b432322562fc1d88abcc15c36978a33dde`이다. 이전 단독 반복과 서로 다른 snapshot이며 이전 SHA와 혼용하지 않는다.

현재 wrapper SHA `053d72a77e50c6cbb9d9182e7a37fc59b5b5be62a66bfad0757cca71d5381a3b`로 freeze했다. result/progress 및20회 raw log/json은 신규 evidence에 보존했다. native guardian suite 자체 병렬은 재현했지만 전체 engine Cargo의 동시 실행/전체 부하 조건은 재현하지 않았다. 원래 engine-first.log의 TerminalUnconfirmed 실패와200ms 경합 가설은 여전히 미확정이며 제품 수정/timeout 증대/typed오류 허용은 없었다.
