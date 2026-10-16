# S03: Windows helper 큰 config native 회귀

- 상태: done (큰 config 실제 정상 전송 범위)
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R02
- 담당/수정 범위: soak_recovery_audit — /tmp/dlpad-helper-large-config-draft.rs, 이 문서
- 선행 작업: actual helper launch 네 회귀 PASS, PIPE_NOWAIT 전송 API

## 계약과 인수

4096-byte pipe보다 충분히 큰 실제 GuardianConfig JSON을 real helper에 보내 Prepared/native creation→durable Bound/unlock→Ack→Begin→Terminal 및 원 receipt까지 검증한다. fixture test binary의 libtest `--skip`에 긴 합성 filter를 전달하여 실행할 fixture는 유지하면서 wire config를 크게 만든다. shell/실제 자격증명/네트워크는 사용하지 않는다. 기존 단순 writer 로직을 복제하지 않는다.

## 검증과 handoff

초안은 tmp만 작성하고 Cargo/native 실행은 root 담당이다. non-consuming reader의 실제 full-pipe cancel 회귀는 현 API만으로 결정적 구성할 수 없다. helper를 임의 PID로 suspend하거나 global 환경 변경으로 우회하지 않는다. root 후속으로 original helper process/thread handle을 통한 fixture-only suspended launch 또는 real helper의 config-read 전 synthetic barrier가 필요하다. 취소 flag를 임의 시간에 바꾸기만 한 race는 full-pipe 관측 증명이 아니다. full-pipe cancel을 미검증으로 남긴다.

tmp fragment SHA256은 `8ff59eaf4bd30410d49b5aadef14f9bf2338ba4c6ed1c339b2f0c07a255b8918`이며 rustfmt 실제 exit 0이다. 기존 launch tests의 read_frame/helper fixture를 재사용한다. 합성 8192-character skip filter로 wire size >32 KiB를 assert하며 실제 성공 후 config bytes 소거·one-shot·Bound digest·원 receipt/reacquire/resolve까지 확인한다. 이 초안은 실제 Windows PASS 결과가 아니다.

Root가 변경 전 Linux nine operations 전체 PASS 및 source SHA351개 일치를 확인한 뒤 freeze를 해제하고 windows_helper_launch.rs tests에 초안을 통합했다. actual native compile/run은 다음 수집에서 확인한다.

## 실제 결과

Root가 windows-helper-large-config-native-first의 raw log/세 operation JSON을 확인했다. engine all-target Clippy, debug guard 선빌드, exact large-config native test 모두 exit0; actual 1 PASS/139 filtered. source_unchanged 및 input_desktop_unchanged 모두 true. >32KiB wire assertion과 실제 Prepared/Bound/ACK/Begin/Terminal0/receipt256/resolve를 통과했다. full-pipe cancel과 변경 후 전체 workspace 검증은 아직 미검증이다.
