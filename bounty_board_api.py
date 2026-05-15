"""Bounty Board API.
Addresses issue #17: Bounty Board API
"""

from fastapi import FastAPI, HTTPException, Query
from pydantic import BaseModel
from typing import Optional, List
from datetime import datetime
from enum import Enum


app = FastAPI(title="Bounty Board API", version="1.0.0")


class BountyStatus(str, Enum):
    open = "open"
    in_progress = "in_progress"
    completed = "completed"
    cancelled = "cancelled"


class BountyCreate(BaseModel):
    title: str
    description: str
    reward: int
    difficulty: str = "medium"


class BountyResponse(BaseModel):
    id: str
    title: str
    description: str
    reward: int
    difficulty: str
    status: BountyStatus
    created_at: str


class BountyListResponse(BaseModel):
    bounties: List[BountyResponse]
    total: int
    page: int


_bounties_db: dict[str, dict] = {}
_counter = 0


@app.post("/bounties", response_model=BountyResponse, status_code=201)
def create_bounty(data: BountyCreate):
    global _counter
    _counter += 1
    bounty = {
        "id": f"bounty_{_counter}",
        "title": data.title,
        "description": data.description,
        "reward": data.reward,
        "difficulty": data.difficulty,
        "status": BountyStatus.open,
        "created_at": datetime.now().isoformat(),
    }
    _bounties_db[bounty["id"]] = bounty
    return bounty


@app.get("/bounties", response_model=BountyListResponse)
def list_bounties(status: Optional[BountyStatus] = None, page: int = Query(1, ge=1), limit: int = Query(20, ge=1, le=100)):
    bounties = list(_bounties_db.values())
    if status:
        bounties = [b for b in bounties if b["status"] == status]
    start = (page - 1) * limit
    return BountyListResponse(
        bounties=bounties[start:start + limit],
        total=len(bounties),
        page=page,
    )


@app.get("/bounties/{bounty_id}", response_model=BountyResponse)
def get_bounty(bounty_id: str):
    bounty = _bounties_db.get(bounty_id)
    if not bounty:
        raise HTTPException(status_code=404, detail="Bounty not found")
    return bounty


@app.patch("/bounties/{bounty_id}/claim", response_model=BountyResponse)
def claim_bounty(bounty_id: str):
    bounty = _bounties_db.get(bounty_id)
    if not bounty:
        raise HTTPException(status_code=404, detail="Bounty not found")
    if bounty["status"] != BountyStatus.open:
        raise HTTPException(status_code=400, detail="Bounty not available")
    bounty["status"] = BountyStatus.in_progress
    return bounty
