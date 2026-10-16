# CPU 실행 계획 GUI 검증

Linux optional CPU snapshot SHA256 `166946017de0b3d0589bf3d02bd864d004d90f3457ff2e314fe5e10a5f865417`의 [최종 raw result](evidence/cpu-plan-boundedtext/result.json)는 exit0/PASS다. [종료 후 source manifest](evidence/cpu-plan-boundedtext/source-inputs.json)를 별도 보관했다. binary는 시험 시작과 종료 SHA가 일치한다. source manifest는 종료 후 수집이며 시험 전후 hash 대조라고 표현하지 않는다.

실행 명령: `python3 scripts/cpu_plan_probe.py --binary target/release/dlpad-cpu --output docs/validation/evidence/cpu-plan-boundedtext --font /mnt/c/Windows/Fonts/malgun.ttf --expected-sha 166946017de0b3d0589bf3d02bd864d004d90f3457ff2e314fe5e10a5f865417`.

private Xvfb/DBus, 임시 app data/미디어 root와 실행하지 않는 합성 도구만 사용했다. 사용자 화면/클립보드/실제 영상/외부 network/계정은 사용하지 않았다. 시작 버튼을 누르지 않았으며 jobs.sqlite 생성과 합성 도구 marker가 모두 없었다.

actual Text.GetText JSON에서 URL query/fragment 전체가 `[원본 URL 생략]`으로 대체됐다. 한글·공백·따옴표·달러가 있는 경로는 한 argv 그대로 유지됐다.720품질,1.001–10.123구간,rate1048576/retries2/fragments4를 확인했고1080변경도 새 JSON에 즉시 반영됐다.143입력에서는 고정 오류와 preview 없음이 확인됐다. 한 페이지의 이전/다음 버튼은 실제 AT-SPI state1107298304로 Enabled/Sensitive bit가 모두 없었다. 닫기 요청부터0.154초에 exit0, owned app/bus/log thread cleanuptrue다.

초기 수집기 실패 raw는 삭제하지 않았다. 첫 실패는 viewport 아래 내용, 두 번째는 공통 accessibility helper의48node 상한이었다. 고급 입력을 추가한 후 공통 widget lookup의 multiline 전체bounds 중심점 조건이 긴 preview를 읽기 대상에서 제외했다. 전용 bounded256node BFS와 직접 Text.GetText로 분리하여 최종 통과했다. 확장 패널의 접근과 계획 헤더 클릭 경로/애니메이션 때문에 생긴 추가 실패들도 보존했다. 일반 버튼은 AT-SPI action, 계획 헤더는 pointer를 사용했다. 계획 헤더의 AT-SPI action/키보드 전체 사용성은 이 결과만으로 확정하지 않는다.

대형 preview의 실제 페이지 이동과256KiB 실패 UI, 다른 OS, 전체 reader/IME, 전 기능 peak 성능은 미검증이다. 순수 builder 경계 시험의 실제 Cargo 결과는 주 담당 통합 기록을 따른다.
