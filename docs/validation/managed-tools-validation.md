# P0 관리 도구 backend 검증

2026-10-01. [직접 다운로드 계약](../work-items/S04-direct-download.md), [관리 선택 계약](../work-items/S04-managed-selection.md), [공급 증거](tool-supply-evidence.md).

직접 executable backend는 현재 yt-dlp에 한정한다. HTTPS만 허용하며 redirect도 HTTPS, proxy 환경 자동 상속 없음, source userinfo/query/fragment 없음, 8192-byte URL·redirect 5·header 64KiB·stream buffer 64KiB·최대256MiB/exact size·120초 phase deadline을 적용한다. guardian `--ignore-config --no-plugin-dirs --version`, 실행 전후 size/identity/SHA256, create-only hardlink publication과 SQLite active 전환을 검증했다. 기존 버전과 외부 설치는 덮어쓰거나 삭제하지 않는다.

Linux/WSL에서 tools fixture 시험은 18/18(1.44초), 실제 공식 pin 2026.08.19 40,446,224 byte HTTPS 다운로드와 임시 소유 설치 시험은 1/1 통과(최종6.47초). 그 앞 재시험 1회는 Http 오류(10.22초)로 실패했으며 정책·시간 상한을 완화하지 않고 같은 시험을 재시도했다. endpoint 실패 원인은 확정하지 않았다. 최종 version argv는 config/plugin 차단을 포함한다. 실제 계정·영상·기존 외부 설치 변경이 없다. 공개 net 시험은 기본 ignored이며 명시 실행 결과만 성공으로 기록한다.

`cargo clippy -p dlpad-engine --all-targets --locked -- -D warnings`, 세 tools Rust 파일 rustfmt check 통과. `cargo test -p dlpad-engine --lib tools::selection::tests --locked` 5/5(1.09초). 전용 fixture는 크기/shortread/hash/HTTP/bodytimeout/pre-cancel/read-cancel/deadline/version/versiontimeout/self-modifying exe/DBtrigger/ownership/symlink/replacement cleanup 실패와 보존을 포함한다.

목록은 metadata-only 최대128개/2048 filesystem entry이며 hash/version의 새로운 증거가 아니다. select_existing와 active_installed는 최대256MiB streaming hash·기록 정확 일치·path/ownership/foreign files/Unix identity를 검사한다. legacy import는 --version 증거가 없다. 파일 path 반환 이후의 실행 직전 pin/검증은 application 통합 계약이다.

blocking HTTP 중 취소는 남은 deadline까지 지연될 수 있으며 OS 파일 I/O를 강제 interrupt하지 않는다. Windows/macOS 실행, Windows handle identity 정리, hardlink 미지원 filesystem 대안, 전체 패키지 license/source/notice 검수와 아래 회복 범위 밖 installer crash 처리는 별도 gate다. [회복 후속](../work-items/S04-staging-recovery.md)은 소유 evidence가 없는 staging을 삭제하지 않고 capacity로 막는 방향이다. std 경로 검사와 unlink 사이 악의적인 동일 사용자 프로세스의 경쟁까지 제거했다고 주장하지 않는다. 전체 S04/P0 출시 완료를 뜻하지 않는다.


[Staging 회복 구현](../work-items/S04-staging-recovery.md)은 2026-10-01 Linux/WSL에서 추가 검증했다. persisted root/directory/file dev+ino, 정확 bytes/SHA256, owner PID/startticks/boot, strict known files가 모두 일치하고 owner가 종료된 경우에만 삭제한다. 무증거/live/unknown/변조/외부 파일은 보존한다. 최대8개 stage·합계512MiB·2048 filesystem entry이며 새 install executable 예상 bytes+64KiB manifest를 먼저 예약한다. 1MiB durable checkpoint 밖 partial bytes는 보존한다.

회복 전용9/9(0.22초), tools 통합27 pass/1 기본 ignored(1.66초), engine all-target clippy 및 네 tools 파일 rustfmt 통과. 실제 owned child SIGKILL/live fixture와 UID1000 chmod PermissionDenied, DB cleanup trigger 실패에서 기존 active/외부 bytes 보존을 시험했다. 새 회복/checkpoint 연결 후 명시한 공식 HTTPS 임시설치1/1(6.48초)이 통과했다. 초기 hash fixture 전체 재시험1회가 실패한 뒤 시험 전용 일반 failure budget1→5초 및 실제 오류 출력 assert로 exact/전체 통과; 최초 반환 오류가 기록되지 않아 원인은 확정하지 않는다. 제품 timeout 변경은 없다.

Linux 밖 owner identity 증거 없는 잔여물은 자동 삭제하지 않는다. final directory publication 중 crash, malicious same-user raw path unlink race, Windows/macOS handle/owner 회복은 잔여 gate다. native Windows 최종 snapshot 시험은 별도 OS 검증 기록을 따른다.


[Transfer lock 회귀 수정](../work-items/S04-transfer-lock-release.md): 주 담당 전체 통합에서 실패2건이 관측되어 실제 오류 출력 후 full engine 반복에서 length16 UnexpectedSize 기대 대신 Io WouldBlock(code11)을 재현했다. Unix flock close만으로 해제하면 clone/fork descriptor가 남는 shared open-file description 때문에 lock이 유지될 수 있다. 명시 unlock RAII와 clone descriptor 재현 시험으로 수정했다. engine111 pass/1ignored 전체8회 연속 통과, 일반 hash fixture1초 원복 및 의도적 version200ms/제품120초 상한 유지, clippy 통과. 이전1→5초 fixture 변경은 최종 해결이 아니며 현재는1초다.
