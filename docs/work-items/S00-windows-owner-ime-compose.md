# S00: Windows owner IME 조합·취소·확정

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R01, R06
- 담당/수정 범위: root는 공유 Cargo/entry/host·통합·실행·이 문서, diagnostic_cache는 /tmp/ime_compose_fixture.rs, metrics_recovery는 /tmp/windows_ime_compose_private_probe.cs 초안만 담당한다. 같은 파일 동시 수정은 없다.
- 선행 작업: S00-windows-private-ime의 실제 owner context 관측 PASS. 최종 스택은 Proposed다.

## 범위와 인수

별도 기본 OFF 명시 fixture에서 앱 소유 context만 사용한다. 고정 합성 한글 문자열의 native SCS_SETSTR와 CPS_CANCEL/CPS_COMPLETE가 실제 winit/egui 조합·확정 이벤트와 포커스 입력으로 이어지는지 확인한다. 실제 물리 키보드 한글 변환 인수와 구별한다. 가짜 WM_IME 주입·전역 키보드·clipboard·layout activation·사용자 desktop fallback은 없다. open 상태 변경이 필요하면 소유 context의 원값을 기록하고 취소·원값 복원·release 결과를 별도로 확인한다. 실패를 성공으로 표시하지 않는다.

원 inherited request/ack/mapping의 제한된 합성 phase 계약, 전체 native deadline, bounded payload/input buffer, UI thread의 blocking/file/pipe I/O 금지를 유지한다. 단계 사이 event loop에 제어를 돌려 실제 native 이벤트가 처리되게 한다. 취소 시 committed 값 보존, 확정 시 정확 합성 문자열, 커서/선택/포커스 및 PNG를 대조한다. 원 PID/creation/HWND/thread와 process Job 정리·input desktop 보존을 root가 독립 검증한다. 아직 초안 단계로 actual 조합 성공은 아니다.

root가 `crates/app/src/ime_compose_fixture.rs`, `accessible_cpu_main.rs`, `software_host.rs`, `crates/app/Cargo.toml`, `scripts/windows_ime_compose_private_probe.cs`, `windows_ime_compose_private_native_probe.py`를 통합했다. `ime-compose-fixture`는 별도 기본OFF feature이며 기존 windows-sys binding을 사용해 새 dependency/version/lock변경은 없다. original UIthread paint/close/fatalexit의 시험context 복원 hook만 추가한다. feature꺼짐에는 hook이컴파일되지 않는다. root가 원cap failure/두phaseACK/복원재시도 eligibility5회귀를 포함한 실제 Windows 전체4검증을 실행 중이다. native 물리키변환과 구별된 실험이고 아직조합통과로표현하지 않는다.

최초 actual Windows 전체4검증과 native capability/state회귀5개가 통과했다. 동일fixture binary7,886,848B/SHA32b7809771d7ccbb326aaff1bd2e0d07c193e7590e61ef813a5fc0d83de826e2에서 원native IMM preedit는두case 모두 정확한글이었고 context release·openfalse복원·원Job자연0/desktop보존을 확인했다. commit은 actual UIA/PNG한글로 통과했다. cancel은 nativecomp제거/API/원상태복원이통과했지만 actual UIA/PNG한글이남아 실패했다. root가원JSON/바이너리/소스/cap/sentinel/화면을대조했고원실패를보존한다. collector의2.5초재관측뒤에도잔류했으며후속actualnativeeventtrace로UIbridge후보원인을검증한다. physicalkeyIME통과로표현하지않는다.
