# S05: 다운로드 상태와 입력 정책 기반

- 상태: implemented-foundation (S00 출시 게이트 대기)
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R05, R07, R08
- 담당/수정 범위: domain 에이전트; crates/core/src/jobs.rs, crates/core/src/options.rs, 이 문서
- 선행 작업: S00 검증 이후 앱 통합; S03/S04 이후 S05 완료

## 사용자 결과

URL·구간·품질과 주요 고급 옵션은 프로세스 시작 전에 검사한다. 재시도는 같은 job에 새 Attempt를 만든다. 완료 상태는 파일 검증 증거가 있어야 한다.

## 계약과 범위

ADR-0002와 INTEGRATIONS 계약을 따르는 순수 정책이다. shell 파싱 없이 OsString argv를 만든다. raw 옵션은 allowlist이며 exec/config/plugin/print/path override를 차단한다. 비밀 인증과 미확인 확장 옵션은 포함하지 않는다. Catalog의 upstream 버전은 아직 검증되지 않음이다. serde 외 새 의존성이 없다.

## 인수 조건

실패·중단·복구 전이, 완료 증거 부재/빈 파일, 중복 attempt, URL 비밀/제어 문자, 시간 overflow, 중복·예약 옵션, 한글/Windows 경로/인용부호를 테스트한다. 작업 ID는 재시도에도 유지된다.

## 검증과 handoff

담당 파일에 rustfmt --edition 2024 실행 성공. rustc --test로 options.rs 4개, jobs.rs 3개 테스트를 실행하여 모두 통과했다(serde는 workspace 빌드 산출물 사용). 통합 담당이 cargo fmt/clippy/test/build를 추가 실행한다. 외부 yt-dlp 고정 버전 argv와 실제 구간 저장은 미검증이다. Artifact 증거는 엔진이 실제 파일을 검증한 후 전달해야 한다. OS UI/IME와 장시간 다운로드는 별도 출시 게이트다.
