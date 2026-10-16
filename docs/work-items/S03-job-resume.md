# S03: 저장된 옵션으로 동일 작업 재개

- 상태: implemented
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08
- 담당/수정 범위: 주 담당 — core/options.rs, engine/download.rs, app/main.rs, app 문자열 catalog, 관련 문서
- 선행 작업: S03-startup-recovery, S05 typed options

## 사용자 결과

복구된 작업을 원래 ID와 검증된 옵션으로 명시적으로 다시 시작한다. 새로운 Attempt를 만들며 자동 실행하지 않는다.

## 계약과 범위

ADR-0002. 주 담당이 SQLite adapter v4 nullable options_json과 버전 1 옵션 DTO를 조정한다. 기존 argv snapshot을 유지하며 두 계약이 일치해야 한다. legacy 옵션 누락과 손상은 추측하지 않고 실패한다. 외부 도구/사용자 파일을 정리하지 않는다. 이후 bounded 이력 UI에 연결한다.

## 인수 조건

옵션 roundtrip으로 argv 동일, 손상/미지원 버전/예약 옵션/잘못된 URL·구간/128KiB 초과 거절. legacy journal 보존. 재개는 Paused/Failed/Interrupted만 허용하고 ID 유지 및 Attempt 증가. 완료·취소·진행 작업 거절. UI 연결과 가상 화면 검증까지 완료 상태로 바꾸지 않는다.

## 검증과 handoff

[통합 검증](../validation/job-resume-validation.md): 116 workspace tests, fmt/clippy/release, 실제 도구 합성 미디어 및 가상 GUI 버튼 재개를 검증했다. Partial 보존 회귀도 통과했다. 새 의존성 없음; JSON은 기존 로컬 argv와 같이 원본 URL을 포함하므로 비밀 저장소나 암호화로 표현하지 않는다.
