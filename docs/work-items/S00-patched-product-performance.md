# S00: 입력 수정 이후 제품 성능 재측정

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R06, R08, S00
- 담당/수정 범위: 주 담당; 본 문서, 전용 측정 JSON/PNG, P0-status.md
- 선행 작업: winit startless patch 및 Linux 공통 통합 검증

## 사용자 결과

한글 입력 후보 수정 이후 기존 제품 목록의 실제 native 크기·RSS·idle CPU를 확인한다.

## 계약과 범위

기존 native_probe.py를 읽기 사용하며 제품 소스 변경 없음. 10000개 합성 목록·Xvfb·기존 로컬 테스트 폰트만 사용한다. 실제 사용자 Documents/화면/영상은 사용하지 않는다. GUI/IME/접근성 인수는 별도다.

## 인수 조건

최신 release SHA와 측정 환경·표본 시간을 남기고 owned 프로세스 정리를 확인한다. 짧은 RSS 표본을 장시간 비용으로 표현하지 않는다.

## 검증과 handoff

`DLPAD_TEST_FONT=/mnt/c/Windows/Fonts/malgun.ttf python3 scripts/native_probe.py target/release/dlpad docs/validation/evidence/S00-patched-iced-library.json --seconds 10` exit0. SHA256 `7e5fe81444a31806189e3a84d51d1ccf809f48bd3c83df1396c9d2fbabacd469`, 9873856 bytes, WSL2/Xvfb software renderer/10000 fixture rows/960×640/10.009초, RSS68404KiB/idle0.0%. owned probe child reaped. [측정 JSON](../validation/evidence/S00-patched-iced-library.json). IME·실제 reader·Windows/macOS·장시간 비용 미검증은 별도 유지한다.
