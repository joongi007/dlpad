# S07: 설치 journal의 명시적 프로세스 중단 보존 시험

- 상태: done (명시적 child 종료 보존 범위)
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root — 공유 process 계약·통합·실행·증거 검토; soak_recovery_audit — /tmp 초안만.
- 선행 작업: journal API/native 인수. CLI readonly 인수와 겹치는 프로젝트/target mutation은 root가 순차 조정한다.

## 사용자 결과

소유 시험 자식이 Prepared 기록을 동기화한 뒤 정상 Drop 없이 종료해도 부모가 남은 prefix를 읽기 전용으로 검사하고 원 파일을 보존한다.

## 계약과 범위

AGENTS/README/PRODUCT/CONTEXT/R11/ADR0005와 기존 process supervisor를 읽는다. 계획 담당 파일은 crates/engine/examples/windows_install_journal_crash_fixture.rs, scripts/windows_msvc_install_journal_crash_collect.py 및 이 기록이다. argv만 사용하며 원 process/Job은 기존 supervisor가 소유한다. child std::process::exit(73)은 이번 작업 소유 fixture만 종료한다. 사용자 WSL/session·다른 프로세스·외부 파일은 조작하지 않는다. 실제 power loss/부모 외부 kill/sleep·자동 recovery/commit/deletion 권한은 이 시험으로 증명하지 않는다.

## 인수 조건

기록 동기화 이후 bounded ready JSON, 명시 exit73, 실제 supervisor 결과를 확인한다. cold inspect는 원 prefix 파일 ID·SHA·namespace를 보존하며 완료 판정/삭제하지 않는다. 타임아웃·취소는 기존 소유 Job 계약에 따른다. exported PID/HANDLE getter 없는 API를 독립 원 HANDLE 증명으로 표현하지 않는다. child output 수집의 내부 상한과 fixture JSON 상한을 구분한다.

## 검증과 handoff

/tmp 초안만 준비됐으며 root review·통합·컴파일/native는 미완료다. 기존 run_metadata의 내부 metadata 상한 8MiB와 fixture ready JSON 4096B 상한을 구분한다. 실제 compile/native와 원 기록 보존을 검증하고 엄밀한 외부 crash/power-loss 인수는 후속으로 남긴다.

root는 초안의 Windows OsString namespace 비교 결함을 확인해 명시 String·고정2파일로 보완했다. 원 ID/size/mtime 전후, 읽기 상한·취소, 단일 inspection deadline, argv localowned 경로와 기존 process::run_metadata 계약을 읽기 대조한 뒤 source를 통합했다. producer 파일은 scripts/windows_msvc_install_journal_crash_example_build.py다. 현재 actual 컴파일/native는 미완료다. metadata 내부8MiB와 ready4096B의 상한 차이는 유지해서 보고한다.

실제 Windows source584 전체4검사819/0/13·example release·native controller1명령이 exit0이다. 내부 child는 Prepared sync/ready 이후 exit73이며 Rust Drop을 우회했다. root 별도2명령으로 원 ID3·두 파일 SHA/canonical chain·외부 sentinel·absent bundle 미생성을 확인했고 독립 raw/pin/bytes 검토를 보관했다. CPU5cf193…8,710,656B·CLIaf1245…896,512B/GNU/lock 보존. root-review와 independent-review는 windows-msvc-install-journal-exit-native-real-20261006-continue에 있다. 외부 parent crash/power loss/실제 session 중단·historical ownership/자동 resume/delete/정식 installer는 증명하지 않는다. 다음은 실제 단계별 journal-aware 새 bundle이다.
