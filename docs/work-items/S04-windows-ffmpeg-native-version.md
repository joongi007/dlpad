# S04: Windows FFmpeg 실제 native version 확인

- 상태: completed
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R11
- 담당/수정 범위: soak_recovery_audit; 본 문서, 신규 scripts/windows_ffmpeg_native_version_probe.py, 신규 evidence만
- 선행 작업: S04-windows-ffmpeg-payload-inventory; 기존 private windows-cli-host

## 사용자 결과

Windows 후보 ffmpeg/ffprobe의 실제 버전과 configuration을 숨김 CLI로 확인한다.

## 계약과 범위

각각 `-version` argv만 실행하며 기존 configure/run host를 재사용한다. 전후 binary SHA/size, private desktop/input desktop 보존, bounded log/deadline를 확인한다. Cargo/기존 코드 변경·visible fallback·실제 영상·계정은 제외한다.

## 인수 조건

실제 Linux 대비 Windows-only enabled는 `pthreads`, `schannel`; Linux-only enabled는 `libdrm`, `libpulse`, `libxcb`, `openssl`, `xlib`이다. 공통 enabled는59개다.

두 명령 실제 exit0, exact version 일치, 원 component SHA/size 보존. workspace 동시 수정 시 source_unchanged 실패를 원시 보존하며 binary 실행 결과와 구분한다. 실제 configuration의 enabled/disabled 목록과 Linux notice 대응 한계를 기록한다. RSS를 추정하지 않는다.

## 검증과 handoff

실제 `PYTHONDONTWRITEBYTECODE=1 python3 scripts/windows_ffmpeg_native_version_probe.py /mnt/d/project/dlpad/docs/validation/evidence/windows-ffmpeg-native-version-first`: exit0. `result.json`, ffmpeg/ffprobe `.log` 및 원 native operation `.json` create-only 보존. 두 명령 각각 exit0/collector failure 없음/private desktop/input desktop unchanged true. 실제 두 버전 모두 `n9.0.2-17-g2a571b6068-20260930`; configuration 한 줄씩 확보했다. 원 binary size/SHA 전후 일치. host 포함 elapsed 약3.29/3.28초, 로그2459/2432 bytes이며 프로그램 startup 시간/RSS는 이 수치에서 추정하지 않는다.

통합 workspace snapshot은 **FAIL 보존**: ffmpeg operation source unchanged true, ffprobe 기간 `crates/engine/src/tools/bounded_zip.rs` 동시 수정으로 false. 이는 실행 binary mutation이나 version 실패가 아니며 root 소스 동결 뒤 전체 snapshot 재검증은 별도 필요하다. collector 결과의 `native_binary_operations_passed=true`, `workspace_snapshot_passed=false`를 합쳐 전체 source 검증 PASS라고 보고하지 않는다.

Windows 실제 enabled/disabled 목록은 `result.json`에 보존했다. Linux 설정을 대신 사용하지 않고 실제 Windows 61 enabled flags를 추출했다. Windows는 schannel 활성, libdrm/libpulse/libxcb/xlib 비활성이다. 공통 feature도 같은 dependency archive가 실제 링크되었다는 증명은 아니므로 기존 Linux notice 수집은 고정 recipe/source 참고 자료에 한정한다. Windows 전체 notice/source 구성·runtime RSS·최소 지원 OS·실제 영상 기능·managed 설치는 미검증이며 supply adoption false다.

`PYTHONPYCACHEPREFIX=/tmp/dlpad-ffmpeg-native-pycache python3 -m py_compile scripts/windows_ffmpeg_native_version_probe.py` 성공. 최초 기본 py_compile는 workspace pycache EROFS로 실패하여 소유 `/tmp` 캐시로 재검증했다. 전원/WSL/desktop/clipboard/제품 Rust/Cargo 변경 없음.
