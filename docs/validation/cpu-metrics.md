# CPU 지표 모듈 구현 범위

신규 accessible_metrics.rs는 기존 engine.metrics volume_free_bytes/sample_network_interface/rate_since를 호출한다. 외부 앱 트래픽을 포함하는 선택 인터페이스만 관측하고 빈 선택은 끈다. engine 미지원 None와 I/O 실패는 측정 불가이며 측정된 실제0과 구분한다. FFmpeg ProcessingFeed 연결은 root 담당이다.

단일 worker와 command/result channel 각각1, UI 최신 설정 snapshot 하나·pending flag·revision으로 이전 루트/인터페이스 결과를 거절한다. full result channel은 새 sample을 버리고 다음1초 관측을 기다리는 bounded 정책이다. 3초 초과 sample은 poll/status 모두 숨긴다. invalid interface는 기존값과 revision을 유지한다. UI 입력은64 chars, 엔진 validator는64 ASCII bytes이며 root는 OS 표현32768bytes·absolute만 수집 대상으로 사용한다. 과대·상대 root는 수집을 끄며 이전 여유숫자를 유지하지 않는다.

수집은 최대초당1회, 볼륨 query 하나와 optional Linux /proc/net/dev 최대128KiB/256rows 하나다. UI thread에는 I/O·join이 없다. idle worker의 recv_timeout은 최대50ms로 stop을 확인하며 active collector는 OS 단일호출 종료를 기다린다. per-second observed context wake와 stale deadline repaint를 사용한다. 단일 syscall의 무한 kernel stall을 timeout으로 강제종료한다고 주장하지 않는다. 비용은 worker 한개/stack 및 제한된 snapshot이며 실제 RSS/CPU 측정은 후속이다.

worker panic은 RAII stopack와 fixed unknown 처리이며 새 선택은 stop 이후 거절한다. invalid/empty selection·루트변경/과대경로·stale·이전revision·panic/stop 회귀5개를 작성했다. 회귀는 실제OS I/O/GUI에 대한 인수가 아니다.

초기 owner는 rustfmt/source freeze와 회귀5개를 root에게 전달했다. 공유 module declaration, Cargo 실행 및 전체 통합 검증은 root 기록을 따른다. 제품 공유 파일/Cargo/schema/catalog/dependency 변경은 없다. 실제3OS/GUI/performance/전체 P0 완료를 주장하지 않는다.


## 2026-10-03 WSL 재시작 뒤 격리 GUI 재검증

[최종 raw result](evidence/cpu-metrics-wsl-recovery-exact/result.json) exit0/PASS, [source hash](evidence/cpu-metrics-wsl-recovery-exact/source-inputs.json) 불변. Linux CPU binary SHA256 `3a1bded7e3e89074f2ba86124074aa32777cae512b81f1b69b4898539ffd728a`. private Xvfb/DBus와 임시 폴더만 사용했고 사용자 데스크톱·클립보드·실제 영상·외부 network installation은 사용하지 않았다.

실행: `python3 scripts/cpu_metrics_probe.py --binary target/release/dlpad-cpu --output docs/validation/evidence/cpu-metrics-wsl-recovery-exact --font /mnt/c/Windows/Fonts/malgun.ttf --expected-sha 3a1bded7e3e89074f2ba86124074aa32777cae512b81f1b69b4898539ffd728a`. 기존 로컬 폰트를 읽기만 했으며 제품 의존성으로 추가하지 않았다.

actual Text.GetText로 `lo`, `../invalid`, 없는 인터페이스, 빈 입력, `lo`를 정확히 확인했다. 빈 입력은 Ctrl+A 뒤 BackSpace를 보내 실제 빈 값으로 만든 뒤 적용했다. 첫 raw 관측 결과는 이 exact 검증 전 기록이라 blank-off 인수 근거로 사용하지 않는다. 중단된 inputfix 디렉터리는 삭제하지 않았으며 재개한 두 fresh 시험이 모두 통과했다.

표시625.5GiB는 실제 statvfs available671621468160bytes와 허용0.1GiB 내 일치한다. lo는 실제0.0 및1.1KiB/s 등 관측값을 표시했고 invalid 입력은 오류와 기존 lo관측을 보존했다. 없는 인터페이스와 빈 선택은 측정 불가였다. 세 반복 관측은 주기적인 업데이트를 증명하지만 정확한 내부 syscall 횟수의 측정은 아니다.

창 닫기 요청부터 정상 exit0까지0.153초, owned app/bus/log thread cleanuptrue 및 binary hash 보존. idle 종료 인수이며 정지한 syscall에서 CloseWait가 지속되는 상황은 강제하지 않았다. 다른 OS, 실제 FFmpeg 진행률, 장시간 stall, 전 기능 peak RSS/CPU 및 전체R01 완료는 이 GUI 결과로 주장하지 않는다.
