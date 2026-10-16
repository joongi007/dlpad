# S04: 관리 보관 버전 선택·제거와 기존 active 실패 보존 GUI

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R05, R08
- 담당/수정 범위: diagnostic_cache agent; 신규 scripts/managed_retained_probe.py, 이 계약, docs/validation/managed-retained-gui-validation.md와 전용 evidence만
- 선행 작업: 관리 직접 설치/선택 backend·native UI 및 확인된 release binary

## 사용자 결과

보관 설치를 실제 GUI에서 active로 선택하고 이전 inactive만 제거하며 외부 파일은 보존한다. 기존 active가 있어도 새 install의 취소/검증 실패가 그 경로·bytes·DB 선택을 바꾸지 않는지 확인한다.

## 계약과 범위

README→PRODUCT→CONTEXT→R05/R08→ADR0003/ARCHITECTURE/INTEGRATIONS를 따른다. 초기 temp managed root/manifest/active DB에 synthetic 두 버전을 seed한다. 이 seed는 GUI ownership/hash/metadata 시험이며 공식 version 검증 통과로 표현하지 않는다. Xvfb만, sibling temp data/media/external fixture만 사용한다. production app/engine/Cargo/공통probe/build 변경 금지.

기존 active 취소는 공식 고정 HTTPS install을 시작하고 staging 관측 뒤 중단한다. hash failure는 주 담당과 합의한 경우에만 소유 download staging의 ELF padding byte를 변조하여 검증 거절을 유도하며 외부 원본·공식 서버 bytes는 변경하지 않는다. 사용자가 제공한 credentials/계정/실제 영상은 사용하지 않는다.

## 인수 조건

실제 inactive 선택→SQLite active 변경→old inactive 제거, new active/외부bytes 보존. 기존 active 취소/HashMismatch 시 DB/path/bytes 불변, staging 종료, native 오류 screenshot. explicit binary SHA와 실제 명령/오류·미검증 범위를 기록한다.

## 검증과 handoff

주 담당이 소유 staging-only ELF padding 변조 계약을 승인했다. 같은 release SHA7e3afa82764e4bc52b254f337db39c81431efede22c7dce9b3bb2aa8aafdfa3d로 실제 select/remove5.518초, 기존 active cancel4.567초, same-size changed SHA reject10.544초 통과. inactive 선택/old 제거와 existing active DB/path/bytes·inactive/external 보존을 확인했다. 임시 staging의 padding9 byte만 변경하며 source/original은 변경하지 않았다. 최종40,446,224 byte와 변조 SHA를 관측했고 UI size/SHA 거절 문구를 직접 검토했다. 정확 Rust discriminant/OCR은 계측하지 않아 hash JSON native_gui_verified=false를 유지한다.

AST/인자9건 통과. [전용 검증 문서](../validation/managed-retained-gui-validation.md)에 명령·actual binary SHA·4JSON/6PNG·초기 관측범위·한계를 기록했다. Synthetic seeded metadata/hash는 공식 version 실행 검증이 아니다. other OS/실제 두 official versions/기존 official active update 취소/영상은 미검증이며 주 담당 독립 검토를 요청한다.
