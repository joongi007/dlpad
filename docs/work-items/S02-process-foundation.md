# S02: 외부 프로세스 실행 기반

- 상태: implemented-foundation (S00 및 앱 통합 대기)
- 분류: feature
- 우선순위: P0
- 요구사항: R01, R04, R08, R13
- 담당/수정 범위: domain 에이전트; crates/engine/src/process.rs, 이 문서
- 선행 작업: S00 검증, S01 저장/목록; 공유 dependency와 module export는 통합 담당

## 사용자 결과

외부 실행 파일과 argv를 별도로 전달한다. stdout/stderr를 동시에 읽고, 보관 로그는 기본 상한 256 KiB 이내, 한 줄은 64 KiB 이내다. 초과 줄·취소·타임아웃은 성공 종료와 구분한다. 정상 종료에도 잔여 자손을 정리하여 pipe drain이 기다리는 상황을 방지한다. 다운로드 완료 여부는 이 실행 결과만으로 판정하지 않는다.

## 계약과 범위

ProcessSpec(executable, args, timeout, max_log_bytes), run(spec, Arc AtomicBool), ExecutionResult(exit_code, logs, truncated, outcome)를 제공한다. shell 문자열 실행은 없다. I/O는 호출자 worker에서 실행해야 하며 UI thread 호출은 금지한다. Unix process group에 SIGTERM 후 최대 200 ms 대기, SIGKILL을 보낸다. Windows는 Job Object kill-on-close 및 트리 종료를 구현했다. 의존성은 Unix libc, Windows windows-sys의 작은 OS API 바인딩으로 통합 담당이 추가한다. 별도 async runtime은 없다. 실행당 drain thread 두 개와 고정 8 KiB 읽기 버퍼 두 개 및 상한 로그를 사용한다.

## 인수 조건

합성 Python fixture로 Unicode/literal argv, 성공과 비정상 종료, stdout/stderr flood, 초과 줄, 취소, 자손 포함 timeout, 실행 파일 부재를 검사한다. pipe 전체를 메모리에 적재하지 않는다. 실제 yt-dlp framing과 파일 검증은 다음 통합 작업이다.

## 검증과 handoff

Linux에서 rustfmt --edition 2024 성공. rustc --test process.rs --extern libc=<workspace rlib>로 6개 테스트 모두 통과했다. fixture 실행 파일은 /usr/bin/python3이며 실제 영상/사용자 Documents/계정에 접근하지 않았다. Cargo workspace 통합 검증은 주 담당이 별도 실행한다.

미검증 및 출시 차단 사항:

- Windows/macOS 실제 실행·취소 검증은 수행하지 않았다.
- Windows std::process spawn부터 Job Object assign까지 짧은 경합 구간이 있다. suspended creation 후 assign/resume 구현 및 시험 전 R08의 완전한 소유권을 주장하지 않는다.
- Linux PR_SET_PDEATHSIG는 직접 자식만 종료한다. 앱 크래시 시 손자 전체 종료는 guardian/감시 파이프 검증이 필요하다. pre_exec syscall은 unsafe 범위와 이유를 코드에 기록했다.
- 의도적으로 setsid로 group을 탈출하는 외부 실행 파일은 현재 소유권 계약 밖이다. 검증한 실행 파일을 대상으로 제한하며 무검증 plugin/exec를 허용하지 않는다.
- S02 제품 완료·8시간 VOD·crash 복구 완료를 뜻하지 않는다.

메타데이터 전용 run_metadata(spec,cancel)는 stdout에 8 MiB 하드 상한을 적용하고 일반 줄 제한과 분리한다. stderr는 기존 bounded tail/줄 제한을 유지한다. 일반 run API와 ProcessSpec은 변경하지 않았다. 큰 JSON·overflow·stderr noise·실행 중 취소는 합성 테스트로 추가 검증했다.
