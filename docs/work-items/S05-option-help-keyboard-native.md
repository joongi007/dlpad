# S05: 다섯 옵션 도움말 키보드 native 인수

- 상태: completed (Linux help5 key activation 범위)
- 분류: validation
- 우선순위: P0
- 요구사항: R07, R01
- 담당 파일: soak_recovery_audit — scripts/cpu_options_probe.py, 이 문서
- 의존성: 기존 도움말 pointer 검증, plan strict keyboard 수집기, root의 immutable CPU binary 및 private Xvfb

## 범위와 인수 조건

`--case help-keyboard`를 추가해 FFmpeg·세션 쿠키·품질·구간·자르기 도움말의 다섯 header에서 GrabFocus 성공, FOCUSED bit 12, Return/space 각각의 실제 expanded open/close, 마지막 설명의 가시 bounds, job/tool 미실행을 검증한다. 상태는 두 uint32를 64bit로 결합하며 EXPANDABLE bit 9와 EXPANDED bit 10을 검사한다. 기존 private Xvfb와 유한 관측 계약을 유지한다. Tab 탐색, screen reader, 다른 OS 검증으로 확대하지 않는다.

도움말 본문으로 스크롤한 뒤에도 header의 실제 FOCUSED가 유지되어야 한다. 닫기는 collapsed와 focused 및 실제 tail node 부재를 모두 요구한다. 접근성 조회 실패나 탐색 상한 초과는 부재로 처리하지 않는다. 모든 Return/space open/close와 job/tool 미실행 검증을 통과한 뒤에만 `keyboard_verified`를 true로 기록한다. `--case all`에도 새 case가 포함된다.

## 실행과 잔여

- 담당 스크립트의 Python `compile(..., 'exec')` 실제 exit 0. GUI와 Cargo는 실행하지 않았다.
- 동결 SHA256: `6aa263dccceb9ea29677c076082c9278e991dbe1fc78472528606b9396640bf8`.
- root 실행 명령: `python3 scripts/cpu_options_probe.py --binary <immutable-cpu> --expected-sha256 <binary-sha256> --output <fresh-evidence> --case help-keyboard`.
- 잔여: root가 private GUI에서 실제 10개 키 open/close, screenshot의 본문 가시성, 원시 상태 및 cleanup을 검증한다. 구현과 Python 문법 검사만으로 native 인수 완료를 주장하지 않는다.

## Root 실제 격리 GUI 인수

`python3 scripts/cpu_options_probe.py --binary /tmp/dlpad-option-help-binary-axhn_dqs/dlpad-cpu --output <canonical-project>/docs/validation/evidence/p0-option-help-keyboard-native-first --expected-sha256 ca8e969f028fdd50f5c7f9ae19c3c71928d7740daa5c93189ec100ee9f5fc77e --case help-keyboard`와 동일 main 호출을 실행했다.

실제 SHA: ca8e969f028fdd50f5c7f9ae19c3c71928d7740daa5c93189ec100ee9f5fc77e. 실제 exit0/result passed=true/keyboard_verified=true/owned_cleanup=true/tool_executed=false. Root가 다섯 header 각각 Return/space 10회 open1124081408/close1124080384와 tailbounds를 원시 JSON으로 독립 대조하고 5개 space-open PNG에서 마지막 문단 가시성을 확인했다. 실행전후 바이너리SHA 동일. Native FOCUSED 인수이며 보이는 focus indicator/Tab-only/reader/Windows/macOS 인수는 아니다.

상태: 해당 Linux key activation 인수 완료. 제품 전체 P0와 Tab/reader는 남는다.
