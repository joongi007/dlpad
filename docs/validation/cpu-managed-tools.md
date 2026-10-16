# CPU 관리 도구 패널 구현 범위

신규 accessible_tools.rs는 기존 ToolManager와 managed::candidate/tool_error를 사용한다. 고정 pin·한국어 오류 매핑을 복제하지 않는다. 관리 yt-dlp만 지원하며 FFmpeg/Deno 설치를 구현했다고 표현하지 않는다. 새 의존성·schema·catalog 변경은 없다.

설치/refresh/활성 선택/보관 제거/active 재검증 후 경로 적용은 하나의 worker에서 수행한다. command/result channel은 각각1, UI busy 동안 추가 요청을 막고 엔진 inventory 상한128을 유지한다. I/O는 show/poll에 없다. 마지막 inventory는 작업 실패시 유지되고 선택 경로는 Use 재검증 성공시만 전달된다. UI remove는 active를 비활성화하며 엔진이 외부 소유 manifest와 unsafe 경로를 다시 거절한다. install 취소는 해당 worker AtomicBool만 변경한다.

request_stop은 새 요청을 막고 설치 취소를 요청한다. 진행 중 단일 작업은 종료 결과를 보낸 뒤 worker가 stop ack한다. 대기 중 명령은 stop 관측 시 실행하지 않는다. 단일 SQLite/파일 호출의 즉시 취소나 네트워크 deadline 이전 강제 종료는 보장하지 않는다. UI thread join/Drop 대기는 없고 worker outer panic은 RAII ack/고정 실패, 작업 panic은 고정 실패 결과다. 실제 UI context가 알려진 요청 완료에는 request_repaint를 사용하고 busy/종료 대기 중만100ms repaint를 요청한다.

5회귀를 작성했다: active없음 경로 미게시 후 worker 재사용, refresh실패 inventory/foreign bytes 보존, disabled/stopping 요청 차단, panic stopack, external manifest 제거 거절 및 외부 bytes 보존. fixture는 독립AtomicU64·create_dir AlreadyExists 최대256retry이며 기존 임시폴더를 덮거나 삭제하지 않는다.

담당 파일 rustfmt exit0, source freeze를 root에게 전달했다. 최초 module 연결/Cargo 검증은 root 담당이며 회귀는 아직 실행하지 않았다. 실제 설치 다운로드·취소/GUI/Windows/macOS/전체 P0 완료를 주장하지 않는다.

## 통합 갱신

최초 clippy에서 cancel 인자를 엔진 계약의 Arc<AtomicBool>로 일치시키고 큰 Outcome::Inventory를 Box로 변경했다. root가 CPU clippy 및59 CPU시험(모듈5회귀 포함) PASS와 release READY를 전달했다. 같은 READY 바이너리의 bounded Linux 실제 GUI는 cpu-managed-tools-gui.md를 따른다. 위 미실행 문장은 초기 작성 상태이며 실제 설치·cancel·3OS 인수는 여전히 남는다.
