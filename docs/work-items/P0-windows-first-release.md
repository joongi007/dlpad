# P0: Windows 최초 릴리스 범위 전환

- 상태: in-progress
- 우선순위: P0
- 담당: root
- 요구사항: R01–R13의 Windows 최초 릴리스 적용 범위
- 수정 파일: README.md, docs/REQUIREMENTS.md, docs/ROADMAP.md, docs/validation/P0-status.md, 이 문서
- 의존성: 기존 공통 Rust 구현 및 Windows GNU native 검증

## 범위와 사용자 결정

사용자 요청으로 최초 릴리스 P0 인수 목표를 Windows로 좁힌다. Linux/macOS 제품 인수와 3OS 출시 게이트는 후속으로 보류한다. 공통 Rust 코드를 삭제하거나 기존 다른OS 증거를 무효화하지 않는다. Linux 개발환경의 공통 단위시험은 가능하지만 Linux/macOS 신규 GUI·패키징 인수는 우선하지 않는다. 이미 실행 중인 Linux 전체collector는 종료 결과를 보존한다.

## 인수 조건과 남은 작업

Windows에서 R01–R13 중 P0 기능·실패복구·취소·경로/권한·관리도구·진단·GUI/IME/접근성·성능·패키징/설치/제거/업그레이드를 실제 검증한다. 관리 실행 pending 실제crash/정리미확인/안전한자동회수, 도구공급notice/source, 실제사이트/EJS 및 절전복귀를 마무리한다. S10 P0는 최초 릴리스에 포함한다. WindowsGNU 성공만으로 확인하지 않은 toolchain/아키텍처/OS버전 지원을 주장하지 않는다. Git CI/CD 후 updater 순서는 유지한다. 공개배포/외부계정/서명결제 등 별도승인 범위는 그대로다.

## 근거와 제한

변경은 사용자 직접 지시다. 최초 릴리스 완료 감사에서 Linux/macOS 미검증을 Windows 출시차단 사유로 사용하지 않지만 후속잔여로 명확히 기록한다. 최종스택의 Windows 실제 GUI/입력/접근성/성능 게이트는 유지한다. 기존 ROADMAP의 3OS 게이트는 후속다중OS 릴리스 목표로 읽는다.
