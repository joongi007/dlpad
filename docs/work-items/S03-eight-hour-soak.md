# S03: 8시간 synthetic headless guardian soak

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R08, R10
- 담당/수정 범위: guardian_fault; 신규 scripts/long_job_soak.py, docs/work-items/S03-eight-hour-soak.md, docs/validation/eight-hour-guardian-soak.md
- 선행 작업: S03-linux-crash-guardian, S03-guardian-drain

## 사용자 결과

실제 8시간 동안 Linux release 앱 headless guardian과 합성 작업·자손을 실행하고 시간별 RSS/CPU/heartbeat/partial 크기 증거를 남긴다. 장시간 관측을 짧은 테스트로 대체하지 않는다.

## 계약과 범위

기존 Rust/Cargo/schema/source를 변경하지 않는다. Linux release app binary의 internal guardian entry만 실행하며 사용자의 desktop·Documents·실제 영상·network를 사용하지 않는다. 새 임시 출력 디렉터리 안의 owned fixture만 만든다. anonymous socket readiness와 소유 pidfd로 제어·정리하고 named waitpid로 회수한다. 전체 process 검색/무관 PID kill, 관리자/cgroup/system 변경을 하지 않는다. Git 정상 저장소가 없어 기존 .git을 보존한다.

## 인수 조건

- 8시간 wall-clock 실제 수행, bounded synthetic partial overwrite와 root/nested/setsid heartbeat 유지.
- 분당 sample JSONL, 전체 ≤512 KiB 및 최대 481 samples, latest 상태와 final 결과 구별.
- 각 소유 프로세스 RSS/CPU와 owner/guardian 비용을 분리한다.
- 짧은 fixture 검증을 먼저 수행하고 8시간 process exec session을 실제 시작한다.
- 시작 직후/중간 실제 elapsed·samples·heartbeat 증거를 보고하며 끝나기 전에 완료로 기록하지 않는다.
- 종료/중단/오류 시 anonymous socket와 pidfd로 fixture만 정리한다.

## 검증과 handoff

하네스 구현·3초 정상/중단/guardianSIGKILL 실패 fixture 검증을 완료했다. 실제 8시간 시험은 exec session 57344로 진행 중이며 첫120.265초/3samples/heartbeat24~25/partial64KiB/guardianRSS4620KiB와CPU0.1165%를 확인했다. [전용 검증 문서](../validation/eight-hour-guardian-soak.md)에 SHA256, 소유 PID, 시작·예상종료·stop.request와 한계를 기록했다. result.json이 없고8시간이 지나지 않았으므로 작업 상태는 in-progress로 유지한다. 본 시험은 headless process guardian longevity 검증이며 실제 yt-dlp/FFmpeg/DB/UI/8시간 media 다운로드·재시작·4GiB fixture·네트워크 복구·sleep/resume 검증은 별도 미완료다. 다운로드 adapter까지 실행하려면 Rust soak runner/fixture 도구 계약을 main 담당자와 별도 합의한다.

## 2026-10-02 재개 시 증거 소실

현재 boot ID794988eb-539b-437e-8396-330738b565bb에서 이전 /tmpstatus와 소유 PID가 존재하지 않는다. 위 running 관측은 당시 중간 기록이며 현재 실행 상태가 아니다. session57344/76382의 8시간 완료 결과는 미검증이다. 기존 시험을 성공으로 처리하지 않는다. 신규 실행과 영구 증거는 S04-soak-restart 작업으로 구분한다.
