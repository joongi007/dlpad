# S03: Linux·WSL interop 외부 도구 소유권 읽기 감사

- 상태: scoped-review-complete
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R05, R08
- 담당/수정 범위: stress_recovery; 이 신규 문서만
- 선행 작업: Linux guardian·사용자 서비스·쿠키 reader receipt 구현

## 사용자 결과

현 Linux 제품의 외부 yt-dlp·명시 FFmpeg 검증에 PE 형식 거절이 없다. 버전 확인을 Windows 커널 자손까지의 소유권 증명으로 해석할 수 없다. 실제 Windows 자손 생존을 재현한 시험은 아니다.

## 계약과 범위

AGENTS, README, PRODUCT, CONTEXT, 관련 REQUIREMENTS, INTEGRATIONS, ADR-0002/0003을 읽었다. Rust·Cargo·기존 시험·시스템 설정은 수정하지 않았다. git status는 정상 저장소가 없다는 기존 상태를 재확인했으며 .git은 보존했다. 사용자 데이터·실제 쿠키·미디어·Windows 자손 실행·신호 시험은 수행하지 않았다.

## 실제 소스 근거

- `crates/app/src/accessible.rs:333`은 yt-dlp 절대 경로를 요구하고 374에서 worker에 전달한다. 형식/확장자 제한은 없다.
- `crates/engine/src/download.rs:579` 이후 yt-dlp를 canonicalize/is_file 후 `--ignore-config --no-plugin-dirs --version`으로 확인한다. PE 검사 없이 버전 문자열을 검증한다.
- 같은 파일 613 이후 명시 FFmpeg도 canonicalize/is_file 후 `-version`의 `ffmpeg version` 토큰을 검증한다. 669/753 이후 yt-dlp에 `--ffmpeg-location`으로 전달한다.
- `crates/core/src/options.rs:330`은 FFmpeg 절대 Unicode 경로·길이·제어 문자·ParentDir만 제한한다. 빈 FFmpeg는 yt-dlp PATH 탐색을 유지한다(INTEGRATIONS 67과 README).
- `crates/engine/src/process.rs:760` 이후 timeout/log 상한은 있지만 Linux guardian 선택 전에 실행 파일 형식 확인은 없다. `process/guardian.rs:165`는 spec.executable을 argv에 전달하고 751은 `Command::new(tool)`로 실행한다.
- `crates/app/src/managed.rs:62`의 공식 Linux 후보는 고정 Linux 자산 yt-dlp_linux와 digest다. 이 특정 공급 경로 자체를 Windows 자산 선택으로 오인하면 안 된다.
- `tools.rs:130`은 manifest 식별자·소유권·digest 모양 등을 확인한다. `tools/selection.rs`의 재선택은 manifest·크기·실행 권한·digest·identity를 확인하며 현 플랫폼/PE 형식을 확인하지 않는다. 로컬 import 시험은 synthetic executable 바이트를 사용한다.

읽은 실행 경로에서 MZ/PE/ELF 검증과 PE 거절 시험을 찾지 못했다. `scripts/managed_retained_probe.py`의 ELF 검사는 시험 조건으로 제품 정책이 아니다. 기존 core의 explicit FFmpeg literal argv 시험과 download의 버전 기록·취소·잘못된 버전 시험은 혼합 커널 소유권 시험이 아니다.

## 결론과 검증 경계

외부 .exe 및 이름을 바꾼 PE 파일은 현재 제품 경로 정책에서 배제되지 않는다. WSL interop가 실행하고 기대 버전 문자열을 출력한다면 현재 확인을 통과할 수 있는 구조다. 실제 해당 파일 실행 또는 Windows 자손 생존은 미검증이다. Linux cgroup·pidfd·boot receipt가 Windows 커널 자손을 소유한다는 근거는 없다. 별도 Windows CLI 시험의 Job Object 보호는 Linux 제품 호출에 자동 적용되지 않는다.

## 후속 구현 제안과 인수 조건

1. Linux 공통 실행 입구에서 최초 version spawn 이전 bounded header 검사로 인식된 PE를 고정 오류로 거절한다. MZ/e_lfanew/PE signature 및 손상·초과 offset을 실패로 처리하고 이름/확장자만으로 판별하지 않는다. symlink canonical 대상과 교체 경합을 다룬다. 현재 Linux Python shebang fixture를 일괄 ELF-only로 차단하지 않는다.
2. 외부/관리 재선택 yt-dlp와 명시 FFmpeg에 동일 정책을 적용한다. FFmpeg는 yt-dlp 자손으로 실행되므로 ProcessSpec 검사만으로 충분하지 않다. 선택 FFmpeg의 검증과 실행 경로 binding이 필요하다.
3. 빈 FFmpeg의 PATH 탐색은 별도 공백이다. 실제 발견된 ffmpeg/ffprobe를 검증하고 한정 경로로 고정하거나 미확인 자동 발견을 실패로 처리하는 정책을 정해야 한다. 명시 경로 PE만 거절한 뒤 PATH까지 해결됐다고 보고하지 않는다.
4. synthetic PE(확장자 없음/대문자 EXE), 짧거나 손상된 header/큰 offset/교체 실패의 version/toolstarted marker 0과 Linux ELF/shebang 정상 대조를 시험한다. 명시 PE FFmpeg 거절은 yt-dlp 다운로드 marker도 0이어야 한다.
5. 네이티브 wrapper가 Windows 도구를 실행하는 경우까지 PE 검사로 완전히 증명할 수 없다. 지원하는 도구·인터프리터·자손 범위를 따로 정한다. 직접 경로 PE 거절은 확인된 공백을 닫는 첫 단계이며 실제 혼합 커널 정리 보장이 아니다.

## 검증과 handoff

2026-10-04 rg/sed 소스 읽기·sha256sum·git status만 수행했다. Cargo/제품 native 실행 0회다. 정상 Git 저장소 없음의 exit128은 보존한다. 문서 쓰기 최초 기본 sandbox는 EROFS였으며 승인된 프로젝트 문서 범위로 다시 작성했다. 시스템 설정 변경이나 제품 우회는 없다.

읽기 시점 SHA-256(전체 제품 검증 snapshot 동일성 주장 없음):

| 파일 | SHA-256 |
| --- | --- |
| engine/download.rs | 9e46d75ff537399ef87121a666e1fbaf566cad7509390ffc789f7253eef57343 |
| engine/process.rs | 27af76ac5c36473ca65adfa9164f288465c9f66801e2b5c1b1b1773ccec26918 |
| engine/process/guardian.rs | 1f11476a949faa6be9b5b36c0b0d77ae5da17f99c412fa10b854cb51ca9b1886 |
| engine/tools/selection.rs | 6a94e364adb7936dd4c4e231a4c880849834823cbecbd6ff6cc0397f76e5d2ec |
| app/managed.rs | 87962fa705d0d934f3a529402ce4a4534e55213a9fda62412444687e9c92c91f |
| core/options.rs | 1d761e942cf4f0656fe9525e62becacf1079206c3a4b478404b79812b4490b7f |

이 감사는 P0 완료나 제안 정책 채택이 아니다.
