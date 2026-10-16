# WSL 종료 뒤 8시간 soak 증거 감사

2026-10-03T11:53:36Z 읽기 전용 host 관측. [원시 증거](evidence/soak-interruption-audit-20261003/observation.json), [작업 계약](../work-items/S04-soak-interruption-audit-20261003.md).

## 판정

기존 boot `794988eb-539b-437e-8396-330738b565bb`에서 현재 `0bc0c937-eee9-4d90-95dc-31a2122b6352`로 변경됐다. 기존 9개 soak PID와 archive PID6202의 /proc 항목은 모두 없다. boot가 다르면 같은 PID/start 값이 있더라도 이전 소유 프로세스로 인정하지 않는다. 이번 관측에는 signal·재시작이 없다.

기존 download·download-runner·guardian의 세 /tmp 폴더는 없다. 보존 evidence/soak-restart-20261002에는 download-result.json/runner-result.json/guardian-result.json/archive-result.json이 모두 없다. 원시 파일을 덮어쓰지 않았으며 archive-status가 기록한 8개 파일의 크기와 SHA는 모두 일치한다.

마지막 archive는 UTC 2026-10-03T03:14:04.657666(한국시간12:14:04)이며 download21531.239초, guardian21517.054초다. 모두28800초 미만이다. root의 [종료 관측](evidence/wsl-interruption-20261003/observation.json)은 한국시간12:14:28 종료와20:48:56 새 부팅을 기록하며 종료 요청자는 unknown이다. 중단 원인으로 특정 사용자·프로그램·에이전트를 확정하지 않는다.

**두 시험은 interrupted, 8시간 완료·성공은 unverified다.** 이전 running JSON은 과거 관측이다. SQLite는 archive 대상이 아니고 원래 임시 폴더가 소실돼 최종 DB reopen, artifact assertion, descendants reap을 확인할 수 없다. 이 중단은 제품의 guardian 실패 시험이나 DB 복구 성공 증거가 아니다.

## 재실행 계약

이 문서의 명령은 복구한 계약이며 이번 감사에서 실행하지 않았다. 현재 소스와 정확한 Rust/Cargo.lock으로 먼저 baseline을 재검증하고 실행 파일을 별도 소유 경로에 고정해야 한다. 기존 launch의 hash를 현재 빌드의 증거로 재사용하지 않는다. root가 Cargo/shared binary를 조정한다.

```sh
cargo build -p dlpad-engine --bin dlpad-process-guard --locked
cargo test -p dlpad-engine --test download_soak --locked -- --nocapture
cargo build -p dlpad --bin dlpad --release --locked
```

`cargo test`가 출력하는 실제 download_soak test executable 경로를 사용한다. 이전 exact 경로는 target/debug/deps/download_soak-eb1ebcee84bd48b1였지만 다음 빌드의 파일명이 같다고 가정하지 않는다. engine은 test executable의 옆 또는 deps 상위에서 dlpad-process-guard를 찾는다. 고정 복사 시 이 배치를 유지하고 두 executable의 SHA를 기록한다. release 앱은 내부 headless guardian entry를 가진다. 실행 뒤 경로를 재빌드로 교체하지 않는 고정 사본이 provenance에 유리하다.

이전 실제 harness argv는 아래와 같았다.

```sh
python3 scripts/download_soak.py target/debug/deps/download_soak-eb1ebcee84bd48b1 /tmp/dlpad-eight-hour-download-20261002-794988eb --seconds 28800
python3 scripts/long_job_soak.py target/release/dlpad /tmp/dlpad-eight-hour-guardian-20261002-794988eb --seconds 28800 --interval 60
```

다음 실행은 현재 boot와 고유 run ID를 이름에 넣은 **새 /tmp 폴더**를 사용한다. download harness는 직접 /tmp 아래 폴더 및 이름에 -runner를 붙인 형제 폴더가 둘 다 없어야 한다. test executable을 argv로 직접 실행하고 DLPAD_DOWNLOAD_SOAK_SECONDS/OUTPUT을 설정한다. guardian harness도 새 폴더만 만들며 DISPLAY/WAYLAND_DISPLAY를 제거해 headless로 실행한다. GUI·실제 영상·Documents·네트워크는 사용하지 않는다.

새 launch는 boot/PID/start, source manifest, test/helper/release SHA, 정확 명령과 session을 함께 기록해야 한다. 다음 archive는 별도 새 evidence에 파일당1MiB/전체8MiB/history512행 상한으로 보관한다. 이전 archive는 임시 inline collector였으며 저장소에는 재사용 가능한 archive script가 없다. 유실된 /tmp collector를 있다고 주장하지 않는다. root가 새 영구 collector를 마련해야 한다. 실제 terminal result와 exitcode, 28800초 이상 elapsed 및 DB/artifact/reap assertion을 직접 대조해야 인수한다.

## 실행 범위와 한계

host Python collector로 /proc boot/stat, 명시적으로 알려진 /tmp 경로 존재 여부, bounded archive JSON/파일 SHA를 조회했다(exit0). 원시 증거 actions=[]/restart_launched=false다. 새 테스트·제품 수정·시스템 변경은 없다. 합성 DownloadRepository+SQLite 및 headless guardian 범위이며 실제 YouTube/치지직 transport/8시간 영상·GUI·절전·다른 OS를 대체하지 않는다.
