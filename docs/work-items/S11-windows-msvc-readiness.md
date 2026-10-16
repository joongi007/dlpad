# S11: Windows MSVC 준비 상태 읽기 전용 조사

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R11
- 담당/수정 범위: soak_recovery_audit; 이 workitem, docs/validation/windows-msvc-readiness.md, 전용 evidence/windows-msvc-readiness-20261003/
- 선행 작업: Windows GNU native CLI hidden host

## 사용자 결과

Windows GNU 검증과 제품 목표 MSVC 준비 상태를 구분하고, 기존 설치 여부와 승인 없이 할 수 있는 다음 단계를 증거로 남긴다.

## 계약과 범위

문서 읽기 순서, R11, ADR-0001/0003과 CICD를 따른다. 기존 환경·hidden host 코드를 읽고 알려진 vswhere/Visual Studio/Windows SDK 경로만 bounded 조회한다. 사용자 desktop은 변경하지 않는다. 설치·다운로드·Cargo/Rust target·시스템 설정·Git 변경은 없다. MSVC 부재면 공식 primary 자료로 설치/권한/크기/라이선스 제약을 조사한다.

## 인수 조건

실제 Windows native 읽기 전용 CLI와 desktop unchanged 결과를 보존한다. cl/link/SDK/native Rust prerequisites를 실제 관측과 자료 추론으로 구분한다. 미검증 MSVC build를 완료로 표시하지 않는다.

## 검증과 handoff

native readonly query+rustc/cargo 버전 조회 모두exit0/private desktop/input unchanged=True. 알려진 MSVC/SDK 설치와 MSVC Rust std는 없고 GNU1.94 구성만 확인했다. 공식 설치는 관리자 권한 및 시스템 drive 변경을 요구해 실행하지 않았다. 공식 SDK NuGet은 부품 후보이며 전체 user-only MSVC toolchain은 미확인이다. 공식출처·정확argv·비용·라이선스/미검증은 전용 validation/evidence에 기록했다. 설치/download/target/Cargo/Git 변경 없음. 실제 MSVC build·GUI·설치·서명은 별도 게이트다.
