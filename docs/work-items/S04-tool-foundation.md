# S04: 검증된 로컬 도구 설치 기반

- 상태: implemented, 통합 검증 대기
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R08
- 담당/수정 범위: library agent — crates/engine/src/tools.rs, 이 문서
- 선행 작업: S02; 현재 격리된 안전 정책·로컬 staging 준비

## 사용자 결과

명시적으로 제공한 로컬 실행 파일을 SHA-256 검증 후 앱 전용 도구 폴더에 설치한다. 검증 실패와 동일 버전 재설치는 기존 활성 버전을 유지한다. 외부 도구·활성 버전·설치 폴더의 예상 밖 파일은 삭제하지 않는다.

## 계약과 범위

ADR-0003, INTEGRATIONS, ARCHITECTURE를 따른다. InstallManifest는 tool/version/target/source_url/sha256/license/ownership/executable을 기록한다. 새 전용 빈 폴더 또는 dlpad owner marker가 있는 폴더만 root로 허용하며 path component symlink와 traversal을 거절한다. 단일 실행 파일 import를 64 KiB 버퍼로 복사·해시 검증하고 실제 bytes를 반환한다. 설치 버전은 create-new로 생성하고 overwrite하지 않는다. SQLite 활성 선택 단일 transaction 뒤 이전 설치를 보존한다. 실패 cleanup은 이번 호출이 만든 두 파일과 빈 폴더만 지우며 recursive deletion하지 않는다. 제거는 root 소유 marker와 저장 manifest를 확인하고 정확히 기록된 inactive binary/manifest만 삭제한다.

Network download, archive extraction, signature 검증, 실제 tool --version 실행과 version pin 사용자 수, FFmpeg/ffprobe set 설치, 공급처·license 승인, UI 통합은 미구현이다. source manifest license 문구를 실제 배포물의 license 검증 결과로 취급하지 않는다. 활성 transaction은 crash-safe pointer 기반이나 imported 파일의 검증 전 process crash 잔여 폴더 recovery는 후속이다. 악의적인 동시 파일시스템 변경까지 차단하는 capability-based openat 구현은 아직 없으며 앱 data root는 사용자 전용 권한으로 격리해야 한다.

## 인수 조건

임시 synthetic 실행 파일의 digest mismatch rollback, 기존 활성 유지, 동일 버전 overwrite 거절, 활성 제거 거절, 외부 도구 제거 거절, traversal/symlink 거절, foreign media 파일 보존. 실제 사용자 미디어와 설치 도구는 테스트에 사용하지 않는다.

## 검증과 handoff

rustfmt 실행 성공. Cargo는 주 담당이 공유 dependency/lock 통합 후 실행한다. Windows/macOS executable 검증과 remove 동작은 실제 OS 미검증. sha2는 streaming hashing CPU 및 dependency binary 비용을 추가하며 실측 전 수치를 주장하지 않는다. 자동 다운로드는 공급처·pin·digest·license가 확정되기 전 활성화하지 않는다.

통합 검토 보완: OS `/tmp`·`/var`와 명시적 선택 ancestor alias는 canonicalization 전에 허용하되 선택된 최종 root symlink와 `..`는 거절한다. 소유 canonical root 아래 marker/lock/install 경로의 symlink 검사는 유지한다. 임시 alias-parent 성공과 final root symlink 거절 regression 테스트를 추가했다. Tool root의 nonempty adoption 거절과 기존 파일 보존은 유지한다. 실제 macOS 실행 검증을 뜻하지 않는다.
