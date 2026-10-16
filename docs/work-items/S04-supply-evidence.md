# S04: 관리 도구 공급처·digest·license 증거

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R05
- 담당/수정 범위: diagnostic_cache agent; docs/work-items/S04-supply-evidence.md, docs/validation/tool-supply-evidence.md, docs/INTEGRATIONS.md
- 선행 작업: S04 관리/외부 소유권 및 streaming digest adapter

## 사용자 결과

도구 자동 설치에 필요한 고정 공급 자료와 검증 한계를 명확히 한다. 공식 release/checksum/signature, FFmpeg 빌드별 license, JS runtime 요구와 비용을 구분한다.

## 계약과 범위

공식 primary source를 확인한다. 공유 Cargo/schema/앱/도구 코드는 변경하지 않는다. FFmpeg 공급 후보를 사용자 확정으로 표현하지 않는다. 외부 설치·계정·사용자 데이터는 변경하지 않는다. 필요 시 이번 작업의 임시 폴더에서 작은 공개 checksum/manifest만 내려받아 검증한다.

## 인수 조건

관측 날짜와 정확한 공식 링크, tag별 URL/digest 검증 방식, signature trust anchor, FFmpeg license/build 조건, JS runtime 조건과 비용, 플랫폼별 실증/미완료를 기록한다. 문자열 URL을 pin 실증으로 취급하지 않는다.

## 검증과 handoff

[공급 검증 기록](../validation/tool-supply-evidence.md)과 INTEGRATIONS에 실제 관측/미완료 계약을 반영했다. 공식 yt-dlp stable 2026.08.19의 tag API/checksum/signature/tag 공개키를 조회하고 격리 GPG에서 예상 fingerprint의 VALIDSIG/exit 0 및 합성 변조 BADSIG/exit 1을 검증했다. 세 OS standalone의 공개 digest/다운로드 byte를 확보했지만 executable bytes를 검증하거나 설치·실행하지 않았다.

BtbN 날짜 고정 Linux LGPL 후보의 작은 checksum 파일을 내려받아 공개 API digest와 대조했다. 공식 Deno v2.9.7 Linux checksum/API 대조와 세 target archive byte를 확인했다. FFmpeg 공급은 후보이며 signature/attestation/source/notice/component digest 및 codec/실행 실증은 남는다. 기존 외부 FFmpeg/ffprobe의 version/license와 launcher 크기는 관측만 했고 수정하지 않았다.

실행/검증:

- stdlib urllib 공개 조회: API 최대 2 MiB+1 byte, checksum/key 최대 64 KiB+1 byte, timeout 30초. 이번 작업 tempdir만 생성·정리.
- GPG `--batch --no-options --homedir <temp/gnupg>` import/fingerprint/verify: 정상 0, 합성 변조 1. 기존 사용자 keyring 사용 없음.
- 문서 내 direct-executable 후보 JSON parse/SHA256 형식/tag URL 확인: 통과.
- `python3 scripts/check_docs.py`: 69 문서와 로컬 링크 통과. 외부 URL/heading checker 검증과 실제 웹조회는 구분.

Git 저장소/commit 없음. 새 의존성·Cargo/schema/앱/도구 코드 변경 없음. 기본 sandbox의 DNS 실패 후 허용된 공개 개발 의존성 조회 escalation을 승인받아 진행했으며 거절 우회는 없었다. 자동 다운로드·압축 해제·관리 설치 UI/통합은 별도 P0 작업이다. archive checksum은 기존 InstallManifest의 executable sha256에 넣지 않는다. Deno Linux/macOS minimum OS와 JS 자원 실측, 세 OS install/solver/codec 실증이 남는다.
